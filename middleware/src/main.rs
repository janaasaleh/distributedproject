use async_std::net::UdpSocket;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;

async fn load_balance(current_server: &mut usize) -> usize {
    *current_server = 1 - *current_server; // Toggle between 0 and 1
    *current_server
}

async fn middleware() {
    let server_addresses = ["0.0.0.0:54321","0.0.0.0:54322"];
    let mut current_server = 0;
    let middleware_address: SocketAddr = "0.0.0.0:12345".parse().expect("Failed to parse middleware address");

    let socket = UdpSocket::bind(&middleware_address).await.expect("Failed to bind middleware socket");
    let mut buffer = [0; 1024];
    let mut ack_buffer = [0; 1024];

    while let Ok((bytes_received, client_address)) = socket.recv_from(&mut buffer).await {
        println!("Yo1");
        let server_index = load_balance(&mut current_server).await;
        println!("Yo2");
        let server_address = server_addresses[server_index];
        let server_address: SocketAddr = server_address.parse().expect("Failed to parse server address");

        let mut server_socket = UdpSocket::bind("0.0.0.0:0").await.expect("Failed to bind server socket");
        server_socket.connect(&server_address).await.expect("Failed to connect to the server");
        println!("Yo3");
        server_socket.send_to(&buffer, &server_address).await.expect("Failed to send data to server");
        println!("Yo4");

        // Receive acknowledgment from the server
        let (ack_bytes_received, _) = server_socket.recv_from(&mut ack_buffer).await.expect("Failed to receive acknowledgment from server");
        println!("Yo5");

        // Send acknowledgment to the client
        socket.send_to(&ack_buffer, client_address).await.expect("Failed to send acknowledgment to client");
        println!("Yo6");

        // Sleep to give time for the server to send the acknowledgment
        sleep(Duration::from_millis(10)).await;

        // Clear the buffer for the next request
        buffer = [0; 1024];
        ack_buffer = [0; 1024];
    }
}

#[tokio::main]
async fn main() {
    middleware().await;
}
