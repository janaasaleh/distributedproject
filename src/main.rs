use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut socket = TcpStream::connect("127.0.0.1:8080").await?;
    println!("Connected to the server.");

    let message = "Hello, server from client 3!";
    socket.write_all(message.as_bytes()).await?;

    let mut response = vec![0u8; message.len()];
    socket.read_exact(&mut response).await?;

    let response_str = String::from_utf8_lossy(&response);
    println!("Server response: {}", response_str);

    Ok(())
}
