use async_std::net::UdpSocket;
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::net::SocketAddr;
use steganography::decoder::*;
use steganography::encoder::*;
use steganography::util::*;

mod big_array;
use big_array::BigArray;

const BUFFER_SIZE: usize = 65536;
const MAX_CHUNCK: usize = 16384;

type PacketArray = [u8; MAX_CHUNCK];

#[derive(Serialize, Deserialize, Debug)]
struct Chunk {
    position: i16,
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

fn remove_trailing_zeros(vec: &mut Vec<u8>) {
    while let Some(&last_element) = vec.last() {
        if last_element == 0 {
            vec.pop();
        } else {
            break;
        }
    }
}

async fn server1(server_address: &str, _middleware_address: &str) {
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
    let mut image_data: Vec<u8> = Vec::new();
    let mut image_chunks = HashMap::<i16, PacketArray>::new();
    let mut packet_number: i16 = 1;

    while let Ok((_bytes_received, _client_address)) = socket.recv_from(&mut buffer).await {
        //sleep(Duration::from_millis(7000)).await;
        // Send the response to the client's middleware

        let packet_string = String::from_utf8_lossy(&buffer[0.._bytes_received]);
        let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();
        shift_left(&mut buffer, _bytes_received);

        // println!("{:?}", deserialized);

        if deserialized.position != -1 {
            image_chunks.insert(deserialized.position, deserialized.packet);
            packet_number += 1;
        } else {
            image_chunks.insert(packet_number, deserialized.packet);
            packet_number = 1;

            let image_chunks_cloned: BTreeMap<_, _> = image_chunks.clone().into_iter().collect();

            for (_key, value) in image_chunks_cloned {
                // println!("Key: {}, Value: {:?}", key, value);
                image_data.extend_from_slice(&value);
            }

            remove_trailing_zeros(&mut image_data);

            if let Ok(image) = image::load_from_memory(&image_data) {
                let (width, height) = image.dimensions();
                println!("Image dimensions: {} x {}", width, height);

                if let Err(err) = image.save("received_image.jpg") {
                    eprintln!("Failed to save the image: {}", err);
                } else {
                    println!("Image saved successfully");
                }
            } else {
                println!("Failed to create image from byte stream");
            }

            let message = "This is a steganography demo!".to_string();
            let payload = str_to_bytes(&message);
            let destination_image = file_as_dynamic_image("encrypt.jpg".to_string());
            let enc = Encoder::new(payload, destination_image);
            let result = enc.encode_alpha();
            save_image_buffer(result, "encrypted.jpg".to_string());

            let encoded_image = file_as_image_buffer("encrypted.jpg".to_string());
            let dec = Decoder::new(encoded_image);
            let out_buffer = dec.decode_alpha();
            let clean_buffer: Vec<u8> = out_buffer.into_iter().filter(|b| *b != 0xff_u8).collect();
            let message = bytes_to_str(clean_buffer.as_slice());

            println!("{}", message);

            // let encryptor = file_as_dynamic_image("encrypt.jpg".to_string());
            // let enc = Encoder::new(&image_data, encryptor);
            // let result = enc.encode_alpha();
            // save_image_buffer(result, "encrypted.jpg".to_string());

            // let encoded_image = file_as_image_buffer("encrypted.jpg".to_string());
            // let dec = Decoder::new(encoded_image);
            // let out_buffer = dec.decode_alpha();

            // println!("{:?}", out_buffer);

            // if let Ok(decoded_image) = image::load_from_memory(&clean_buffer) {
            //     let (width, height) = decoded_image.dimensions();
            //     println!("Image dimensions: {} x {}", width, height);

            //     if let Err(err) = decoded_image.save("decoded.jpg") {
            //         eprintln!("Failed to save the image: {}", err);
            //     } else {
            //         println!("Image saved successfully");
            //     }
            // } else {
            //     println!("Failed to create image from byte stream");
            // }
        }

        socket
            .send_to(
                "Sent acknowledgement to middleware".as_bytes(),
                _client_address,
            )
            .await
            .expect("Couldnt send to middleware");
    }

    // let _ = fs::write("image.png", &image_data);
}

async fn server_middleware(middleware_address: &str, server_addresses: Vec<&str>) {
    let middleware_socket = UdpSocket::bind(middleware_address)
        .await
        .expect("Failed to bind middleware socket");

    // let server_to_server_socket = UdpSocket::bind("127.0.0.2:8080")
    //     .await
    //     .expect("Failed to bind server to server socket");

    println!("Server middleware is listening on {}", middleware_address);
    let mut _current_server: i32 = 0;
    let mut receive_buffer = [0; BUFFER_SIZE];
    let mut send_buffer = [0; BUFFER_SIZE]; // Separate buffer for sending data
    while let Ok((bytes_received, client_address)) =
        middleware_socket.recv_from(&mut receive_buffer).await
    {
        // server_to_server_socket
        //     .connect("127.0.0.3:8080")
        //     .await
        //     .expect("Failed to connect to the server");
        // server_to_server_socket
        //     .connect("127.0.0.4:8080")
        //     .await
        //     .expect("Failed to connect to the server");

        // let index: &[u8] = &current_server.to_be_bytes();

        // server_to_server_socket
        //     .send_to(index, "127.0.0.3:8080")
        //     .await
        //     .expect("Failed to send index to server 2");
        // server_to_server_socket
        //     .send_to(index, "127.0.0.4:8080")
        //     .await
        //     .expect("Failed to send index to server 3");

        // if current_server == 0 {
        //     current_server = 0;
        // } else if current_server == 1 {
        //     current_server = 0;
        // } else if current_server == 2 {
        //     current_server = 0;
        // }

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

        let (ack_bytes_received, _server_address) = server_socket
            .recv_from(&mut receive_buffer)
            .await
            .expect("Failed to receive acknowledgment from server");

        // Send the acknowledgment from the server to the client's middleware
        middleware_socket
            .send_to(&receive_buffer[..ack_bytes_received], client_address)
            .await
            .expect("Failed to send acknowledgment to client");
        shift_left(&mut receive_buffer, ack_bytes_received);

        // Clear the receive buffer for the next request
        receive_buffer = [0; BUFFER_SIZE];
    }
}

// async fn register_user(
//     client_socket: UdpSocket,
//     dos_address: &str,
//     username: &str,
//     usertype: &str,
// ) {
//     let registration_message = format!("REGISTER:{}:{}", username, usertype);
//     client_socket
//         .send_to(registration_message.as_bytes(), dos_address)
//         .await
//         .expect("Failed to send registration request");
//     let mut response_buffer = [0; 1024];
//     let (bytes_received, _dos_address) = client_socket
//         .recv_from(&mut response_buffer)
//         .await
//         .expect("Failed to receive response");
//     let response = String::from_utf8_lossy(&response_buffer[..bytes_received]);
//     println!("Registration response: {}", response);
// }

// async fn query_online_users(client_socket: UdpSocket, middleware_address: &str) {
//     // Send a query message to request the list of online users
//     client_socket
//         .send_to("QUERY".as_bytes(), middleware_address)
//         .await
//         .expect("Failed to send query request");
//     let mut response_buffer = [0; 1024];
//     let (bytes_received, _middleware_address) = client_socket
//         .recv_from(&mut response_buffer)
//         .await
//         .expect("Failed to receive response");
//     let response = String::from_utf8_lossy(&response_buffer[..bytes_received]);
//     println!("Online users: {}", response);
// }

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "127.0.0.2:21112"
        .parse()
        .expect("Failed to parse middleware address");
    let middleware_address_str = middleware_address.to_string();

    // Define the server addresses and middleware addresses
    let server_addresses = ["127.0.0.2:54321", "127.0.0.3:54322", "127.0.0.4:54323"];
    let server1_task = server1("127.0.0.2:54321", &middleware_address_str);

    // Start the server middleware
    let server_middleware_task =
        server_middleware(&middleware_address_str, server_addresses.to_vec());

    let _ = tokio::join!(server1_task, server_middleware_task);
}
