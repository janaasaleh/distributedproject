use async_std::net::UdpSocket;
use std::fs;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;

// async fn load_balance(current_server: &mut usize) -> usize {
//     *current_server = 1 - *current_server; // Toggle between 0 and 1
//     *current_server
// }1024

const BUFFER_SIZE: usize = 102400;

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "127.0.0.8:12345"
        .parse()
        .expect("Failed to parse middleware address");

    // Create a UDP socket for the client
    let client_socket = UdpSocket::bind("127.0.0.8:0")
        .await
        .expect("Failed to bind client socket");

    // Create a UDP socket for the middleware
    let middleware_socket = UdpSocket::bind(&middleware_address)
        .await
        .expect("Failed to bind middleware socket");

    let server_addresses = ["127.0.0.2:21112", "127.0.0.3:21111", "127.0.0.4:21113"];
    let mut buffer = [0; BUFFER_SIZE];
    let mut ack_buffer = [0; BUFFER_SIZE];

    let middleware_task = tokio::spawn(async move {
        if let Ok((_bytes_received, client_address)) =
            middleware_socket.recv_from(&mut buffer).await
        {
            println!("Yo1");
            println!("Yo2");
            let server_socket = UdpSocket::bind("127.0.0.8:0")
                .await
                .expect("Failed to bind server socket");
            for server_address in &server_addresses {
                let server_address: SocketAddr = server_address
                    .parse()
                    .expect("Failed to parse server address");
                server_socket
                    .connect(&server_address)
                    .await
                    .expect("Failed to connect to the server");
                println!("Yo3");
                server_socket
                    .send_to(&buffer, &server_address)
                    .await
                    .expect("Failed to send data to server");
                println!("Yo4");

                // Set a timeout for acknowledgment
                let timeout_duration = Duration::from_secs(1); // Set a 2-second timeout
                let ack_timeout = tokio::time::timeout(
                    timeout_duration,
                    server_socket.recv_from(&mut ack_buffer),
                )
                .await;

                if let Ok(ack_result) = ack_timeout {
                    match ack_result {
                        Ok((_ack_bytes_received, _)) => {
                            println!("Yo5"); // Acknowledgment received
                        }
                        Err(_) => {
                            // Timeout occurred, but this will not block for the full 2 seconds
                            println!(
                                "Timeout: No acknowledgment received from server {}",
                                server_address
                            );
                        }
                    }
                } else {
                    continue;
                }
            }
            middleware_socket
                .send_to(&ack_buffer, client_address)
                .await
                .expect("Failed to send acknowledgment to client");
            println!("Yo6");

            // Sleep to give time for the server to send the acknowledgment
            sleep(Duration::from_millis(10)).await;

            // Clear the buffer for the next request
            buffer = [0; BUFFER_SIZE];
            ack_buffer = [0; BUFFER_SIZE];
        }
    });

    // Client code here
    let image_data = fs::read("image1.jpg").expect("Failed to read the image file");
    let middleware_address = "127.0.0.8:12345"; // Replace with the actual middleware address and port
                                                //sleep(Duration::from_millis(5000)).await;
    client_socket
        .send_to(&image_data, middleware_address)
        .await
        .expect("Failed to send request to middleware");

    // Receive response from the server (the first one)
    let mut client_buffer = [0; BUFFER_SIZE];
    let (_bytes_received, _) = client_socket
        .recv_from(&mut client_buffer)
        .await
        .expect("Failed to receive response from server");

    if let Err(err) = fs::write("encrypted.jpg", &client_buffer[.._bytes_received]) {
        eprintln!("Failed to write the received image to file: {}", err);
    } else {
        println!("Received image saved to encrypted.jpg");
    }

    client_buffer = [0; BUFFER_SIZE];

    // Wait for the middleware task to finish
    middleware_task.await.expect("Middleware task failed");
}
