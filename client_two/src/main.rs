use async_std::net::UdpSocket;
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use steganography::decoder::*;
use steganography::util::*;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{sleep, timeout};

mod big_array;
use big_array::BigArray;

const BUFFER_SIZE: usize = 65536;
const MAX_CHUNCK: usize = 16384;

type PacketArray = [u8; MAX_CHUNCK];

#[derive(Serialize, Deserialize, Debug)]
struct Chunk {
    total_packet_number: usize,
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

async fn middleware_task(middleware_socket: UdpSocket) {
    let server_addresses = ["127.0.0.2:21112", "127.0.0.3:21111", "127.0.0.4:21113"];
    //let server_addresses = ["127.0.0.2:21112"];
    let mut buffer = [0; BUFFER_SIZE];
    let mut ack_buffer = [0; BUFFER_SIZE];
    //let middleware_address: SocketAddr = "127.0.0.9:12345".parse().expect("Failed to parse middleware address");

    // let vector = 0
    // let mut code_zero: String = String::new();
    // code_zero = "AA".to_string();
    let mut var=0;
    let mut serverar=0;

    while let Ok((_bytes_received, client_address)) = middleware_socket.recv_from(&mut buffer).await
    {
        var+=1;
        if client_address.ip().to_string() == "127.0.0.9" {
            println!("Client:{}",var);
            let _server_socket = UdpSocket::bind("127.0.0.9:0")
                .await
                .expect("Failed to bind server socket");
            for server_address in &server_addresses
            {
                let server_address: SocketAddr = server_address
                    .parse()
                    .expect("Failed to parse server address");
                middleware_socket
                    .send_to(&buffer[0.._bytes_received], &server_address)
                    .await
                    .expect("Failed to send data to server");
            }
                shift_left(&mut buffer, _bytes_received);
                let timeout_duration = Duration::from_secs(12);
                match timeout(
                    timeout_duration,
                    middleware_socket.recv_from(&mut ack_buffer),
                ).await
                {
                    Ok(Ok((ack_bytes_received, _server_address))) => {
                        let ack_string=format!("Ack {} sent to client",var);
                        middleware_socket
                            .send_to(ack_string.as_bytes(), client_address)
                            .await
                            .expect("Failed to send acknowledgment to client");
                        shift_left(&mut ack_buffer, ack_bytes_received);
                    }
                    Err(_) => {
                        // code_zero = "".to_string();
                        // middleware_socket
                        //     .send_to(code_zero.as_bytes(), client_address)
                        //     .await
                        //     .expect("Failed to send acknowledgment to client");
                        // code_zero = "AA".to_string();
                    }
                    Ok(Err(_e)) => {

                    }
                }
           
        } else {
            serverar+=1;
            println!("Server:{}",serverar);
            let my_client_address="127.0.0.9:3411";
            middleware_socket
                .send_to(&buffer[0.._bytes_received], my_client_address)
                .await
                .expect("Failed to send acknowledgment to client");
            middleware_socket
                .send_to(&buffer[0.._bytes_received], client_address)
                .await
                .expect("Failed to send acknowledgment to client");
            shift_left(&mut ack_buffer, _bytes_received);
        }

        // Sleep to give time for the server to send the acknowledgment
        sleep(Duration::from_millis(10)).await;

        // Clear the buffer for the next request
        buffer = [0; BUFFER_SIZE];
        ack_buffer = [0; BUFFER_SIZE];
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
//     let mut response_buffer = [0; BUFFER_SIZE];
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
//     let mut response_buffer = [0; BUFFER_SIZE];
//     let (bytes_received, _middleware_address) = client_socket
//         .recv_from(&mut response_buffer)
//         .await
//         .expect("Failed to receive response");
//     let response = String::from_utf8_lossy(&response_buffer[..bytes_received]);
//     println!("Online users: {}", response);
// }

#[tokio::main]
async fn main() {
    let dos_address = "127.0.0.255:12345";
    let middleware_address: SocketAddr = "127.0.0.9:12345"
        .parse()
        .expect("Failed to parse middleware address");
    let client_socket = UdpSocket::bind("127.0.0.9:3411")
        .await
        .expect("Failed to bind client socket");
    //let client_socket_register = UdpSocket::bind("127.0.0.9:8090").await.expect("Failed to bind client socket");
    //let client_socket_query = UdpSocket::bind("127.0.0.9:8091").await.expect("Failed to bind client socket");
    //register_user(client_socket_register,dos_address, "Client1","Client").await;
    //println!("Finished Registry");
    //query_online_users(client_socket_query,dos_address).await;
    let middleware_socket = UdpSocket::bind(&middleware_address)
        .await
        .expect("Failed to bind middleware socket");

    tokio::spawn(middleware_task(middleware_socket));
    let _dos_address_clone = dos_address; // Assuming `dos_address` is defined elsewhere
    let termination = Arc::new(Mutex::new(0));
    let termination_clone = Arc::clone(&termination);

    let mut signal = signal(SignalKind::interrupt()).expect("Failed to create signal handler");
    tokio::spawn(async move {
        signal.recv().await;
        println!("Received termination signal");

        //let unregister_message = "UNREGISTER";
        //let dos_socket = UdpSocket::bind("127.0.0.9:9000").await.expect("Failed to bind socket");
        //dos_socket
        //    .send_to(unregister_message.as_bytes(), dos_address_clone)
        //    .await
        //    .expect("Failed to send unregister message");
        //
        // Notify other tasks waiting for the signal
        //let _ = tx.send(());
        *termination_clone.lock().unwrap() = 1;

        // Exit the application
        std::process::exit(0);
        //return;
    });

    while *termination.lock().unwrap() == 0 {
        println!("Press Enter to send a Request");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        if input.trim() == "" {
            let mut client_buffer = [0; BUFFER_SIZE];

            println!("Sending Image");
            let image_data = fs::read("image.png").expect("Failed to read the image file");
            let middleware_address = "0.0.0.0:12345"; // Replace with the actual middleware address and port
                                                      //sleep(Duration::from_millis(5000)).await;

            let packet_number = (image_data.len() / MAX_CHUNCK)+1;
            for (index, piece) in image_data.chunks(MAX_CHUNCK).enumerate() {
                let is_last_piece = index == packet_number-1;
                let chunk = Chunk {
                    total_packet_number: packet_number,
                    position: if is_last_piece {
                        -1
                    } else {
                        index.try_into().unwrap()
                    },
                    packet: {
                        let mut packet_array = [0; MAX_CHUNCK];
                        packet_array[..piece.len()].copy_from_slice(piece);
                        packet_array
                    },
                   
                };
                let serialized = serde_json::to_string(&chunk).unwrap();

                client_socket
                    .send_to(&serialized.as_bytes(), middleware_address)
                    .await
                    .expect("Failed to send piece to middleware");

                if index !=packet_number-1
                {
                    let (num_bytes_received, _)=client_socket
                    .recv_from(&mut client_buffer)
                    .await
                    .expect("Failed to receive acknowledgement from server");
                let received_string = String::from_utf8_lossy(&client_buffer[..num_bytes_received]);
                println!("Received {}",received_string);
                }
            }
            println!("Finished All Packets");
            println!("{}",packet_number);

            let mut encrypted_image_data: Vec<u8> = Vec::new();
            let mut image_chunks = HashMap::<i16, PacketArray>::new();
            let mut j=0;
            let mut enecrypted_image_packet_number=packet_number;


            while j<enecrypted_image_packet_number {
                if let Ok((_bytes_received, _client_address)) =
                    client_socket.recv_from(&mut client_buffer).await
                {
                    let packet_string = String::from_utf8_lossy(&client_buffer[0.._bytes_received]);
                    let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();
                    enecrypted_image_packet_number=deserialized.total_packet_number;
                    let mio=deserialized.position;
                    println!("EIPN {}",mio);
                    shift_left(&mut client_buffer, _bytes_received);
                    if j == enecrypted_image_packet_number-1 {
                        image_chunks.insert(enecrypted_image_packet_number.try_into().unwrap(), deserialized.packet);
                        println!("Ana 5aragt {}",j);
                    } else {
                        image_chunks.insert(deserialized.position, deserialized.packet);
                    }
                    j+=1;
                }
            }
            println!("Ana 5aragt");
            let image_chunks_cloned: BTreeMap<_, _> = image_chunks.clone().into_iter().collect();

            for (_key, value) in image_chunks_cloned {
                // println!("Key: {}, Value: {:?}", key, value);
                encrypted_image_data.extend_from_slice(&value);
            }
           

            remove_trailing_zeros(&mut encrypted_image_data);
            println!("EID Size {}",encrypted_image_data.len());

            if let Ok(encrypted_image) = image::load_from_memory(&encrypted_image_data) {
                let (width, height) = encrypted_image.dimensions();
                println!("Image dimensions: {} x {}", width, height);

                if let Err(err) = encrypted_image.save("encrypted.png") {
                    eprintln!("Failed to save the image: {}", err);
                } else {
                    println!("Image saved successfully");
                }
            } else {
                println!("Failed to create image from byte stream");
            }


            let encoded_image = file_as_image_buffer("encrypted.png".to_string());
            let dec = Decoder::new(encoded_image);
            let out_buffer = dec.decode_alpha();
            let clean_buffer: Vec<u8> = out_buffer.into_iter().filter(|b| *b != 0xff_u8).collect();
            let message = bytes_to_str(clean_buffer.as_slice());



            let decoded_image_data = base64::decode(message).unwrap_or_else(|e| {
                eprintln!("Error decoding base64: {}", e);
                Vec::new()
            });

            if let Ok(decoded_image) = image::load_from_memory(&decoded_image_data) {
                let (width, height) = decoded_image.dimensions();
                println!("Image dimensions: {} x {}", width, height);

                if let Err(err) = decoded_image.save("decoded.png") {
                    eprintln!("Failed to save the image: {}", err);
                } else {
                    println!("Image saved successfully");
                }
            } else {
                println!("Failed to create image from byte stream");
            }
        }
    }
}
