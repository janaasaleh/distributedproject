use async_std::net::UdpSocket;
use std::fs;
use std::io::Error;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;

const BUFFER_SIZE: usize = 140000;

const MAX_PACKET_SIZE: usize = 1400;

async fn send_image(
    client_socket: &UdpSocket,
    image_data: &[u8],
    destination: &SocketAddr,
    max_packet_size: usize,
) -> Result<(), Error> {
    for chunk in image_data.chunks(max_packet_size) {
        // Send a chunk of the image data
        client_socket
            .send_to(chunk, destination)
            .await
            .expect("Failed to send image chunk");
    }

    Ok(())
}

async fn receive_image(
    client_socket: &UdpSocket,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut image_data: Vec<u8> = Vec::new();
    let mut buffer = [0; BUFFER_SIZE];

    loop {
        match client_socket.recv_from(&mut buffer).await {
            Ok((bytes_received, _)) => {
                // Append the received data to the image_data vector
                image_data.extend_from_slice(&buffer[..bytes_received]);
            }
            Err(_) => {
                // No more data to receive, exit the loop
                break;
            }
        }
    }

    // Write the reconstructed image data to a file
    fs::write(output_path, &image_data)?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "127.0.0.8:12345"
        .parse()
        .expect("Failed to parse middleware address");

    let client_socket = UdpSocket::bind("127.0.0.8:0")
        .await
        .expect("Failed to bind client socket");

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
                server_socket
                    .send_to(&buffer, &server_address)
                    .await
                    .expect("Failed to send data to server");

                let timeout_duration = Duration::from_secs(1);
                let ack_timeout = tokio::time::timeout(
                    timeout_duration,
                    server_socket.recv_from(&mut ack_buffer),
                )
                .await;

                if let Ok(ack_result) = ack_timeout {
                    match ack_result {
                        Ok((_ack_bytes_received, _)) => {
                            println!("Achnowledgment received");
                        }
                        Err(_) => {
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

            sleep(Duration::from_millis(10)).await;

            buffer = [0; BUFFER_SIZE];
            ack_buffer = [0; BUFFER_SIZE];
        }
    });

    let image_data = fs::read("image1.jpg").expect("Failed to read the image file");

    if let Err(err) = send_image(
        &client_socket,
        &image_data,
        &middleware_address,
        MAX_PACKET_SIZE,
    )
    .await
    {
        eprintln!("Failed to send the image: {}", err);
    } else {
        println!("Image sent successfully to middleware");
    }

    if let Err(err) = receive_image(&client_socket, "encrypted.jpg").await {
        eprintln!("Failed to receive and save the image: {}", err)
    } else {
        println!("Received image saved to 'encrypted.jpg'");
    }

    middleware_task.await.expect("Middleware task failed");
}
