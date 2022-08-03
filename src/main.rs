use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
/// Continuously accept new incoming connections on localhost:8080, and for each new connection,
/// spawn a new task that reads lines from the connection and broadcasts them to all other connections
async fn main() {
    // Create TcpConnection on localhost:8080
    let listener = TcpListener::bind("localhost:8080").await.unwrap();

    // Create broadcast channel for 10 clients
    let (tx, _rx) = broadcast::channel(10);

    // Continuously run as until app closed
    loop {
        //Accept new incoming connections
        let (mut socket, addr) = listener.accept().await.unwrap();

        // Due to scope, clone tx and rx variables
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        // Spawning a new task that reads lines from the connection and broadcasts them to all other connections
        tokio::spawn(async move {
            // Splitting the socket into a reader and a writer.
            let (reader, mut writer) = socket.split();

            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            // A loop that is waiting for either a tx or a rx to complete.
            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        tx.send((line.clone(), addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, other_addr) = result.unwrap();

                        if addr != other_addr {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
