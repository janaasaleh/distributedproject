use async_std::net::UdpSocket;
use std::net::UdpSocket as StdUdpSocket;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;
use std::thread;

async fn load_balance(current_server: &mut usize) -> usize {
    *current_server = 1 - *current_server; // Toggle between 0 and 1
    *current_server
}

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "0.0.0.0:12345".parse().expect("Failed to parse middleware address");

    // Create a UDP socket for the client
    let client_socket = UdpSocket::bind("0.0.0.0:0").await.expect("Failed to bind client socket");

    // Create a UDP socket for the middleware
    let middleware_socket = UdpSocket::bind(&middleware_address).await.expect("Failed to bind middleware socket");

    let server_addresses = ["0.0.0.0:21112", "0.0.0.0:21111"];
    let mut current_server = 1;
    let mut buffer = [0; 1024];
    let mut ack_buffer = [0; 1024];

    let middleware_task = tokio::spawn(async move {
        if let Ok((bytes_received, client_address)) = middleware_socket.recv_from(&mut buffer).await {
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
            middleware_socket.send_to(&ack_buffer, client_address).await.expect("Failed to send acknowledgment to client");
            println!("Yo6");

            // Sleep to give time for the server to send the acknowledgment
            sleep(Duration::from_millis(10)).await;

            // Clear the buffer for the next request
            buffer = [0; 1024];
            ack_buffer = [0; 1024];
        }
    });

    // Client code here
    let client_message = "Request from Client 2!";
    let middleware_address = "0.0.0.0:12345"; // Replace with the actual middleware address and port
    //sleep(Duration::from_millis(5000)).await;
    client_socket.send_to(client_message.as_bytes(), middleware_address).await.expect("Failed to send request to middleware");

    // Receive response from the server
    let mut client_buffer = [0; 1024];
    client_socket.recv_from(&mut client_buffer).await.expect("Failed to receive response from server");
    let response = String::from_utf8_lossy(&client_buffer);
    println!("Client received response from server: {}", response);

    // Wait for the middleware task to finish
    middleware_task.await.expect("Middleware task failed");
}