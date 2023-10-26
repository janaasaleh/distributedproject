use async_std::net::UdpSocket;
use async_std::task;
use std::net::SocketAddr;

async fn server1(server_address: &str, middleware_address: &str) {
    let parts: Vec<&str> = server_address.split(':').collect();
    let port = parts[1].parse::<u16>().expect("Failed to parse port as u16");
    let server_address: SocketAddr = server_address.parse().expect("Failed to parse server address");

    let socket = UdpSocket::bind(&server_address).await.expect("Failed to bind server socket");
    println!("Server 1 socket is listening on {}", server_address);

    let mut buffer = [0; 1024];

    while let Ok((bytes_received, client_address)) = socket.recv_from(&mut buffer).await {
        let message = String::from_utf8_lossy(&buffer);
        println!("Server 1 received: {}", message);

        let response = match port {
            54322 => "Server 1 received your message",
            _ => "Server 1 received your message",
        };

        println!("Server 1 responding with: {}", response);
       
        // Send the response to the client's middleware
        if let Err(err) = socket.send_to(response.as_bytes(), client_address).await {
            eprintln!("Server 1 failed to send acknowledgment to middleware: {}", err);
        }
        // Clear the buffer for the next request
        buffer = [0; 1024];
    }
}

async fn server_middleware(middleware_address: &str, server_addresses: Vec<&str>) {
    let middleware_socket = UdpSocket::bind(middleware_address).await.expect("Failed to bind middleware socket");
    println!("Server middleware is listening on {}", middleware_address);

    let mut receive_buffer = [0; 1024];
    let mut send_buffer = [0; 1024]; // Separate buffer for sending data
    while let Ok((bytes_received, client_address)) = middleware_socket.recv_from(&mut receive_buffer).await {
        println!("Entered Here 1");
        let server_index = 0;  // You can implement load balancing logic here
        let server_address = server_addresses[server_index];
        let server_address: SocketAddr = server_address.parse().expect("Failed to parse server address");

        let mut server_socket = UdpSocket::bind("0.0.0.0:0").await.expect("Failed to bind server socket");
        server_socket.connect(&server_address).await.expect("Failed to connect to the server");
       
        // Copy the received data to the send buffer
        send_buffer[..bytes_received].copy_from_slice(&receive_buffer[..bytes_received]);

        server_socket.send_to(&send_buffer[..bytes_received], &server_address).await.expect("Failed to send data to server");
        println!("Entered Here 2");


        let (ack_bytes_received, _) = server_socket.recv_from(&mut receive_buffer).await.expect("Failed to receive acknowledgment from server");
        println!("Entered Here 3");

        // Send the acknowledgment from the server to the client's middleware
        middleware_socket.send_to(&receive_buffer[..ack_bytes_received], client_address).await.expect("Failed to send acknowledgment to client");
        println!("Entered Here 4");

        // Clear the receive buffer for the next request
        receive_buffer = [0; 1024];
    }
}


#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "0.0.0.0:21112".parse().expect("Failed to parse middleware address");
    let middleware_address_str = middleware_address.to_string();
   
    // Define the server addresses and middleware addresses
    let server_addresses = ["0.0.0.0:54321", "0.0.0.0:54322"];
    let server1_task = server1("0.0.0.0:54321", &middleware_address_str);
   
    // Start the server middleware
    let server_middleware_task = server_middleware(&middleware_address_str, server_addresses.to_vec());
   
    let _ = tokio::join!(server1_task, server_middleware_task);
}