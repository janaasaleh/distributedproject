use async_std::net::UdpSocket;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::SocketAddr;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::Duration;

mod big_array;
use big_array::BigArray;

const BUFFER_SIZE: usize = 10240;
const MAX_CHUNCK: usize = 256;

type PacketArray = [u8; MAX_CHUNCK];

#[derive(Serialize, Deserialize, Debug)]
struct Chunk {
    position: i32,
    #[serde(with = "BigArray")]
    packet: PacketArray,
}

fn shift_left(array: &mut [u8; BUFFER_SIZE], positions: usize) {
    let len = array.len();

    // Ensure positions is within array bounds
    if positions < len {
        // Copy elements from position `positions` to the beginning of the array
        for i in 0..len - positions {
            array[i] = array[i + positions];
        }
        // Set the remaining positions to default values
        for i in len - positions..len {
            array[i] = Default::default();
        }
    }
}

async fn server1(server_address: &str, _middleware_address: &str) {
    let mut image_data: Vec<u8> = Vec::new();
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
    let mut image_chunks = HashMap::<i32, PacketArray>::new();

    while let Ok((_bytes_received, _client_address)) = socket.recv_from(&mut buffer).await {
        //sleep(Duration::from_millis(7000)).await;
        // Send the response to the client's middleware

        let packet_string = String::from_utf8_lossy(&buffer[0.._bytes_received]);
        let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();
        shift_left(&mut buffer, _bytes_received);

        println!("{:?}", deserialized);

        // image_chunks.insert(deserialized.position, deserialized.packet);

        // image_data.extend_from_slice(deserialized.packet);
    }

    let _ = fs::write("image.png", &image_data);
    buffer = [0; BUFFER_SIZE];
}

async fn server_middleware(middleware_address: &str, server_addresses: Vec<&str>) {
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
            current_server = 0;
        } else if current_server == 1 {
            current_server = 0;
        } else if current_server == 2 {
            current_server = 0;
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
            .send_to(&send_buffer[0..bytes_received], &server_address)
            .await
            .expect("Failed to send data to server");
        shift_left(&mut send_buffer, bytes_received);

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

async fn register_user(
    client_socket: UdpSocket,
    dos_address: &str,
    username: &str,
    usertype: &str,
) {
    let registration_message = format!("REGISTER:{}:{}", username, usertype);
    client_socket
        .send_to(registration_message.as_bytes(), dos_address)
        .await
        .expect("Failed to send registration request");

    let mut response_buffer = [0; 1024];
    let (bytes_received, _dos_address) = client_socket
        .recv_from(&mut response_buffer)
        .await
        .expect("Failed to receive response");
    let response = String::from_utf8_lossy(&response_buffer[..bytes_received]);
    println!("Registration response: {}", response);
}

async fn query_online_users(client_socket: UdpSocket, middleware_address: &str) {
    // Send a query message to request the list of online users
    client_socket
        .send_to("QUERY".as_bytes(), middleware_address)
        .await
        .expect("Failed to send query request");

    let mut response_buffer = [0; 1024];
    let (bytes_received, _middleware_address) = client_socket
        .recv_from(&mut response_buffer)
        .await
        .expect("Failed to receive response");
    let response = String::from_utf8_lossy(&response_buffer[..bytes_received]);
    println!("Online users: {}", response);
}

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "127.0.0.2:21112"
        .parse()
        .expect("Failed to parse middleware address");
    let middleware_address_str = middleware_address.to_string();

    let dos_address = "127.0.0.255:12345";
    let server_socket_register = UdpSocket::bind("127.0.0.2:8090")
        .await
        .expect("Failed to bind client socket");
    let server_socket_query = UdpSocket::bind("127.0.0.2:8091")
        .await
        .expect("Failed to bind client socket");
    register_user(server_socket_register, dos_address, "Server1", "Server").await;
    println!("Finished Registry");
    query_online_users(server_socket_query, dos_address).await;

    // Define the server addresses and middleware addresses
    let server_addresses = ["127.0.0.2:54321", "127.0.0.3:54322", "127.0.0.4:54323"];
    let server1_task = server1("127.0.0.2:54321", &middleware_address_str);

    // Start the server middleware
    let server_middleware_task =
        server_middleware(&middleware_address_str, server_addresses.to_vec());

    let dos_address_clone = dos_address.to_string();

    // Spawn a task to listen for the termination signal (Ctrl+C)
    let mut signal =
        signal(SignalKind::interrupt()).expect("Failed to register Ctrl+C signal handler");

    // Spawn a task to listen for the termination signal (Ctrl+C)
    tokio::spawn(async move {
        signal.recv().await;
        println!("Received Ctrl+C signal, cleaning up and terminating...");

        // Send an "UNREGISTER" message
        let unregister_message = "UNREGISTER";
        let dos_socket = UdpSocket::bind("127.0.0.2:9001")
            .await
            .expect("Failed to bind socket");
        dos_socket
            .send_to(unregister_message.as_bytes(), dos_address_clone)
            .await
            .expect("Failed to send unregister message");

        // Sleep briefly to ensure the message is sent
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Exit the application
        std::process::exit(0);
    });

    let _ = tokio::join!(server1_task, server_middleware_task);
}
