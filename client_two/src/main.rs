// client2.rs
use std::net::UdpSocket;

fn main() {
    let middleware_address = "0.0.0.0:12345"; // Replace with the actual middleware address and port

    // Create a UDP socket for the client
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind client socket");

    // Send a message to the middleware
    let message = "Request from Client 2!";
    socket.send_to(message.as_bytes(), middleware_address).expect("Failed to send request to middleware");

    // Receive response from the server
    let mut buffer = [0; 1024];
    socket.recv_from(&mut buffer).expect("Failed to receive response from server");
    let response = String::from_utf8_lossy(&buffer);
    println!("Client 2 received response from server: {}", response);
}
