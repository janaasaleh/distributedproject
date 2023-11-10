use async_std::net::UdpSocket;
use std::io::Error;
use std::net::SocketAddr;

const BUFFER_SIZE: usize = 140000;

const MAX_PACKET_SIZE: usize = 1400;

async fn send_image(
    socket: &UdpSocket,
    image_data: &[u8],
    destination: &SocketAddr,
    max_packet_size: usize,
) -> Result<(), Error> {
    for chunk in image_data.chunks(max_packet_size) {
        // Send a chunk of the image data
        socket
            .send_to(chunk, destination)
            .await
            .expect("Failed to send image chunk");
    }

    Ok(())
}

async fn server1(server_address: &str, _middleware_address: &SocketAddr) {
    let parts: Vec<&str> = server_address.split(':').collect();
    let _port = parts[1]
        .parse::<u16>()
        .expect("Failed to parse port as u16");
    let server_address: SocketAddr = server_address
        .parse()
        .expect("Failed to parse server address");

    let socket = UdpSocket::bind(&server_address)
        .await
        .expect("Failed to bind server socket");
    println!("Server 1 socket is listening on {}", server_address);

    let mut buffer = [0; BUFFER_SIZE];

    let mut image_chunks: Vec<u8> = Vec::new();

    while let Ok((_bytes_received, client_address)) = socket.recv_from(&mut buffer).await {
        let chunk = &buffer[.._bytes_received];
        image_chunks.extend_from_slice(chunk);

        if chunk.len() < BUFFER_SIZE {
            let image_result = image::load_from_memory(&image_chunks);
            if let Ok(mut image) = image_result {
                if let Err(err) =
                    send_image(&socket, &image_chunks, _middleware_address, MAX_PACKET_SIZE).await
                {
                    eprintln!(
                        "Server 1 failed to send processed image to middleware: {}",
                        err
                    );
                } else {
                    println!("Server 1 sent processed image back to middleware");
                }
            } else {
                eprintln!("Failed to load received image");
            }
            image_chunks.clear();
        }

        if let Err(err) = socket
            .send_to(&buffer[.._bytes_received], client_address)
            .await
        {
            eprintln!(
                "Server 1 failed to send acknowledgment to middleware: {}",
                err
            );
        }
        println!("Middleware address {}", client_address);
        // Clear the buffer for the next request
        buffer = [0; BUFFER_SIZE];
    }
}

async fn server_middleware(middleware_address: &SocketAddr, server_addresses: Vec<&str>) {
    let middleware_socket = UdpSocket::bind(middleware_address)
        .await
        .expect("Failed to bind middleware socket");

    let server_to_server_socket = UdpSocket::bind("127.0.0.2:8080")
        .await
        .expect("Failed to bind server to server socket");

    println!("Server middleware is listening on {}", middleware_address);
    let mut current_server: i32 = 0;
    let mut receive_buffer = [0; BUFFER_SIZE];
    let mut send_buffer = [0; BUFFER_SIZE]; // Separate buffer for sending data
    while let Ok((bytes_received, client_address)) =
        middleware_socket.recv_from(&mut receive_buffer).await
    {
        println!("Entered Here 1");

        server_to_server_socket
            .connect("127.0.0.3:8080")
            .await
            .expect("Failed to connect to the server");
        server_to_server_socket
            .connect("127.0.0.4:8080")
            .await
            .expect("Failed to connect to the server");

        let index: &[u8] = &current_server.to_be_bytes();

        server_to_server_socket
            .send_to(index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 2");
        server_to_server_socket
            .send_to(index, "127.0.0.4:8080")
            .await
            .expect("Failed to send index to server 3");

        if current_server == 0 {
            current_server += 1;
        } else if current_server == 1 {
            current_server += 1;
            continue;
        } else if current_server == 2 {
            current_server = 0;
            continue;
        }

        //continue;
        let server_index = 0; // You can implement load balancing logic here
        let server_address = server_addresses[server_index];
        let server_address: SocketAddr = server_address
            .parse()
            .expect("Failed to parse server address");

        let server_socket = UdpSocket::bind("127.0.0.2:0")
            .await
            .expect("Failed to bind server socket");
        server_socket
            .connect(&server_address)
            .await
            .expect("Failed to connect to the server");

        // Copy the received data to the send buffer
        send_buffer[..bytes_received].copy_from_slice(&receive_buffer[..bytes_received]);

        server_socket
            .send_to(&send_buffer[..bytes_received], &server_address)
            .await
            .expect("Failed to send data to server");
        println!("Entered Here 2");

        let (ack_bytes_received, server_caddress) = server_socket
            .recv_from(&mut receive_buffer)
            .await
            .expect("Failed to receive acknowledgment from server");
        println!("Entered Here 3");
        println!("Server address {}", server_caddress);

        // Send the acknowledgment from the server to the client's middleware
        middleware_socket
            .send_to(&receive_buffer[..ack_bytes_received], client_address)
            .await
            .expect("Failed to send acknowledgment to client");
        println!("Entered Here 4");

        // Clear the receive buffer for the next request
        receive_buffer = [0; BUFFER_SIZE];
    }
}

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "127.0.0.2:21112"
        .parse()
        .expect("Failed to parse middleware address");

    // Define the server addresses and middleware addresses
    let server_addresses = ["127.0.0.2:54321", "127.0.0.3:54322", "127.0.0.4:54323"];
    let server1_task = server1("127.0.0.2:54321", &middleware_address);

    // Start the server middleware
    let server_middleware_task = server_middleware(&middleware_address, server_addresses.to_vec());
    let _ = tokio::join!(server1_task, server_middleware_task);
}
