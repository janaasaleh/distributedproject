use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::io;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server is listening on 127.0.0.1:8080...");

    while let Ok((mut socket, _)) = listener.accept().await {
        let mut buffer = [0; 1024];
        while let Ok(n) = socket.read(&mut buffer).await {
            if n == 0 {
                println!("Client disconnected.");
                break; // Exit the loop when the client disconnects
            }

            let received_data = &buffer[..n];
            let message = String::from_utf8_lossy(received_data);
            println!("Received: {}", message);

            // Respond to the client
            let response = "Message received by the server\n";
            if let Err(e) = socket.write_all(response.as_bytes()).await {
                eprintln!("Failed to write to socket: {}", e);
                break;
            }
        }
    }

    Ok(())
}
