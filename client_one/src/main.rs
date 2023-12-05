use async_std::net::UdpSocket;
use chrono::prelude::*;
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::{self, Cursor};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use steganography::decoder::Decoder;
use steganography::encoder::Encoder;
use steganography::util::*;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::{sleep, timeout};

mod big_array;
use big_array::BigArray;

const BUFFER_SIZE: usize = 65536;
const MAX_CHUNCK: usize = 16384;
static mut VIEWS: u8 = 10;

type PacketArray = [u8; MAX_CHUNCK];
#[derive(Serialize, Deserialize, Debug)]
struct Chunk {
    views: u8,
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

fn delete_image(file_path: &str) {
    match fs::remove_file(file_path) {
        Ok(_) => println!("Image deleted successfully."),
        Err(err) => eprintln!("Error deleting image: {}", err),
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
#[derive(Debug, Deserialize)]
struct User {
    address: String,
    name: String,
    user_type: String,
}
async fn client_listener(client_listener_socket: UdpSocket) {
    let mut buffer = [0; BUFFER_SIZE];
    let mut ack_buffer = [0; BUFFER_SIZE];
    while let Ok((_bytes_received, client_address)) =
        client_listener_socket.recv_from(&mut buffer).await
    {
        let message_user = String::from_utf8_lossy(&buffer[.._bytes_received]);
        if(message_user.starts_with("VIEWS"))
        {
            let mut p=true;
            while(p)
            {
                println!("Do you want to give this client more views?");
                println!("1.Yes");
                println!("2.No");
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");
                if(input=="1")
                {
                    p=false;
                    println!("How many views to give them?");
                    let mut input = String::new();
                    // Read a line from standard input
                    io::stdin().read_line(&mut input).expect("Failed to read line");
                
                    // Trim whitespace and convert the string to a slice of bytes
                    let index: &[u8] = input.trim().as_bytes();
                    client_listener_socket.send_to(index,client_address);

                }
                else if(input=="2")
                {
                    p=false;
                    let index: &[u8] = "0".trim().as_bytes();
                    client_listener_socket.send_to(index,client_address);
                }
                else
                {

                }
            }
            continue;
        }


        let png_files: Vec<String> = fs::read_dir("my_images/")
            .expect("Failed to read directory")
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.path().extension().and_then(|e| e.to_str()) == Some("png") {
                        Some(e.file_name().to_string_lossy().into_owned())
                    } else {
                        None
                    }
                })
            })
            .collect();
        let png_file_legnth: &[u8] = &png_files.len().to_be_bytes();
        println!("Len:{:?}", png_file_legnth);
        client_listener_socket
            .send_to(png_file_legnth, client_address)
            .await
            .expect("Failed to send data to server");
        let mut iterate = 0;
        while iterate < png_files.len() {
            println!("{}", &png_files[iterate]);
            let img = image::open(format!(
                "my_images/{}",
                &png_files[iterate].trim_matches('"')
            ))
            .expect("Failed to open image");
            let low_res = img.resize(512, 512, image::imageops::FilterType::Nearest);

            // let image_data = fs::read(&png_files[iterate]).expect("Failed to read the image file");
            let mut image_data = Vec::new();
            low_res
                .write_to(
                    &mut Cursor::new(&mut image_data),
                    image::ImageOutputFormat::Png,
                )
                .expect("Failed to write image to byte steam");

            let packet_number = (image_data.len() / MAX_CHUNCK) + 1;
            println!("Packets:{}", packet_number);
            for (index, piece) in image_data.chunks(MAX_CHUNCK).enumerate() {
                let is_last_piece = index == packet_number - 1;
                let chunk = Chunk {
                    views: unsafe { VIEWS },
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
                client_listener_socket
                    .send_to(&serialized.as_bytes(), client_address)
                    .await
                    .expect("Failed to send piece to middleware");
                let (num_bytes_received, _) = client_listener_socket
                    .recv_from(&mut ack_buffer)
                    .await
                    .expect("Failed to receive acknowledgement from server");

                //let received_string = String::from_utf8_lossy(&client_buffer[..num_bytes_received]);
                //println!("Received {}", received_string);
            }
            iterate += 1;
        }
        let png_files_json =
            serde_json::to_vec(&png_files).expect("Failed to serialize png_files to JSON");
        client_listener_socket
            .send_to(&png_files_json, client_address)
            .await
            .expect("Failed to send piece to client");
        let (num_bytes_received, _) = client_listener_socket
            .recv_from(&mut buffer)
            .await
            .expect("Failed to receive acknowledgement from server");
        let filename = format!(
            "my_images/{}",
            String::from_utf8_lossy(&buffer[..num_bytes_received]).to_string()
        );
        let (num_bytes_received, _) = client_listener_socket
            .recv_from(&mut buffer)
            .await
            .expect("Failed to receive acknowledgement from server");
        let requested_views = u8::from_le_bytes([buffer[7]]);
        println!("RVs:{}", requested_views);

        //Start Encryption
        let image_data = fs::read(filename).expect("Failed to read the image file");
        let middleware_address = "127.0.0.8:12345"; // Replace with the actual middleware address and port
                                                    //sleep(Duration::from_millis(5000)).await;

        let packet_number = (image_data.len() / MAX_CHUNCK) + 1;
        for (index, piece) in image_data.chunks(MAX_CHUNCK).enumerate() {
            let is_last_piece = index == packet_number - 1;
            let chunk = Chunk {
                views: requested_views,
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
            println!("Index:{}", index);
            let serialized = serde_json::to_string(&chunk).unwrap();

            client_listener_socket
                .send_to(&serialized.as_bytes(), middleware_address)
                .await
                .expect("Failed to send piece to middleware");

            let (num_bytes_received, _) = client_listener_socket
                .recv_from(&mut buffer)
                .await
                .expect("Failed to receive acknowledgement from server");
            let received_string = String::from_utf8_lossy(&buffer[..num_bytes_received]);
            println!("Received {}", received_string);
        }
        println!("Finished All Packets");
        println!("{}", packet_number);

        let mut encrypted_image_data: Vec<u8> = Vec::new();
        let mut image_chunks = HashMap::<i16, PacketArray>::new();
        let mut j = 0;
        let mut enecrypted_image_packet_number = packet_number;

        while j < enecrypted_image_packet_number {
            println!("j : {}  EIPN: {}", j, enecrypted_image_packet_number);
            if let Ok((_bytes_received, _client_address)) =
                client_listener_socket.recv_from(&mut buffer).await
            {
                let packet_string = String::from_utf8_lossy(&buffer[0.._bytes_received]);
                let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();
                enecrypted_image_packet_number = deserialized.total_packet_number;
                shift_left(&mut buffer, _bytes_received);

                if j == enecrypted_image_packet_number - 1 {
                    image_chunks.insert(
                        enecrypted_image_packet_number.try_into().unwrap(),
                        deserialized.packet,
                    );
                    println!("Ana 5aragt {}", j);
                } else {
                    image_chunks.insert(deserialized.position, deserialized.packet);
                }
                j += 1;
                client_listener_socket
                    .send_to("Ack".as_bytes(), _client_address)
                    .await
                    .expect("Failed to send");
            }
        }
        println!("Ana 5aragt");
        let image_chunks_cloned: BTreeMap<_, _> = image_chunks.clone().into_iter().collect();

        for (_key, value) in image_chunks_cloned {
            // println!("Key: {}, Value: {:?}", key, value);
            encrypted_image_data.extend_from_slice(&value);
        }

        remove_trailing_zeros(&mut encrypted_image_data);
        println!("EID Size {}", encrypted_image_data.len());

        //End of Encryption

        let packet_number = (encrypted_image_data.len() / MAX_CHUNCK) + 1;
        for (index, piece) in encrypted_image_data.chunks(MAX_CHUNCK).enumerate() {
            let is_last_piece = index == packet_number - 1;
            let chunk = Chunk {
                views: requested_views,
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
            println!("Index:{}", index);
            let serialized = serde_json::to_string(&chunk).unwrap();

            client_listener_socket
                .send_to(&serialized.as_bytes(), client_address)
                .await
                .expect("Failed to send piece to middleware");

            let (num_bytes_received, _) = client_listener_socket
                .recv_from(&mut buffer)
                .await
                .expect("Failed to receive acknowledgement from server");
            let received_string = String::from_utf8_lossy(&buffer[..num_bytes_received]);
            println!("Received {}", received_string);
        }
    }
}

async fn middleware_task(middleware_socket: UdpSocket) {
    let server_addresses = ["127.0.0.2:21112", "127.0.0.3:21111", "127.0.0.4:21113"];
    //let server_addresses = ["127.0.0.2:21112"];
    let mut buffer = [0; BUFFER_SIZE];
    let mut ack_buffer = [0; BUFFER_SIZE];
    //let middleware_address: SocketAddr = "127.0.0.8:12345".parse().expect("Failed to parse middleware address");

    // let vector = 0
    // let mut code_zero: String = String::new();
    // code_zero = "AA".to_string();
    let mut var = 0;
    let mut serverar = 0;
    let _server_socket = UdpSocket::bind("127.0.0.8:0")
        .await
        .expect("Failed to bind server socket");
    let mut current_server = "".to_string();

    while let Ok((_bytes_received, client_address)) = middleware_socket.recv_from(&mut buffer).await
    {
        var += 1;
        if client_address.ip().to_string() == "127.0.0.8" {
            println!("Client:{}", var);
            let packet_string = String::from_utf8_lossy(&buffer[0.._bytes_received]);
            let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();
            if deserialized.position == 0 {
                for server_address in &server_addresses {
                    let server_address: SocketAddr = server_address
                        .parse()
                        .expect("Failed to parse server address");
                    middleware_socket
                        .send_to(&buffer[0.._bytes_received], &server_address)
                        .await
                        .expect("Failed to send data to server");
                }
            } else {
                let server_address: SocketAddr = current_server
                    .parse()
                    .expect("Failed to parse server address");
                middleware_socket
                    .send_to(&buffer[0.._bytes_received], server_address)
                    .await
                    .expect("Failed to send data to server");
            }
            shift_left(&mut buffer, _bytes_received);
            let timeout_duration = Duration::from_secs(120);
            match timeout(
                timeout_duration,
                middleware_socket.recv_from(&mut ack_buffer),
            )
            .await
            {
                Ok(Ok((ack_bytes_received, _server_address))) => {
                    let ack_string = format!("Ack {} sent to client", var);
                    middleware_socket
                        .send_to(ack_string.as_bytes(), client_address)
                        .await
                        .expect("Failed to send acknowledgment to client");
                    shift_left(&mut ack_buffer, ack_bytes_received);
                    current_server = _server_address.to_string();
                }
                Err(_) => {
                    // code_zero = "".to_string();
                    // middleware_socket
                    //     .send_to(code_zero.as_bytes(), client_address)
                    //     .await
                    //     .expect("Failed to send acknowledgment to client");
                    // code_zero = "AA".to_string();
                }
                Ok(Err(_e)) => {}
            }
        } else {
            serverar += 1;
            println!("Server:{}", serverar);
            let my_client_address = "127.0.0.8:8085";
            middleware_socket
                .send_to(&buffer[0.._bytes_received], my_client_address)
                .await
                .expect("Failed to send acknowledgment to client");
            middleware_socket
                .recv_from(&mut ack_buffer)
                .await
                .expect("Failed");
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

async fn register_user(client_socket: UdpSocket, username: &str, usertype: &str) {
    let server_addresses = ["127.0.0.2:21112", "127.0.0.3:21111", "127.0.0.4:21113"];
    let registration_message = format!("REGISTER:{}:{}", username, usertype);
    for server_address in &server_addresses {
        client_socket
            .send_to(registration_message.as_bytes(), server_address)
            .await
            .expect("Failed to send registration request");
    }
}
async fn query_online_users(client_socket: UdpSocket) {
    // Send a query message to request the list of online users
    let server_addresses = ["127.0.0.2:21112", "127.0.0.3:21111", "127.0.0.4:21113"];
    for server_address in &server_addresses {
        client_socket
            .send_to("QUERY".as_bytes(), server_address)
            .await
            .expect("Failed to send query request");
    }
    let mut response_buffer = [0; BUFFER_SIZE];
    let (bytes_received, _middleware_address) = client_socket
        .recv_from(&mut response_buffer)
        .await
        .expect("Failed to receive response");
    let response = String::from_utf8_lossy(&response_buffer[..bytes_received]);
    println!("Online users: {}", response);
}

async fn query_online_users_to_send(
    client_socket: UdpSocket,
    username: String,
) -> std::string::String {
    // Send a query message to request the list of online users
    let server_addresses = ["127.0.0.2:21112", "127.0.0.3:21111", "127.0.0.4:21113"];
    let mut sender_address = "".to_string();
    for server_address in &server_addresses {
        client_socket
            .send_to("QUERY2".as_bytes(), server_address)
            .await
            .expect("Failed to send query request");
    }
    let mut response_buffer = [0; BUFFER_SIZE];
    let (bytes_received, _middleware_address) = client_socket
        .recv_from(&mut response_buffer)
        .await
        .expect("Failed to receive response");
    let users: Vec<User> = serde_json::from_slice(&response_buffer[..bytes_received])
        .expect("Failed to deserialize users");

    // Filter users based on the username condition
    let filtered_users: Vec<_> = users.iter().filter(|user| user.name != username).collect();

    // Display enumerated names of filtered users
    for (index, user) in filtered_users.iter().enumerate() {
        println!("{}: {}", index + 1, user.name);
    }

    // Ask the user to choose an option
    println!("Choose a user by entering its number:");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    let chosen_index: usize = input.trim().parse().expect("Invalid input");

    // Check if the index is valid
    if chosen_index > 0 && chosen_index <= filtered_users.len() {
        // Construct the target address
        let chosen_user = &filtered_users[chosen_index - 1];
        sender_address = format!("{}:8085", chosen_user.address);
        //let target_address: SocketAddr = format!("{}:12345", chosen_user.address)
        //    .parse()
        //    .expect("Failed to parse target address");
        // Send a message to the chosen user
        //let message = "Hello, chosen user!";
        //client_socket
        //    .send_to(message.as_bytes(), &target_address)
        //    .await
        //    .expect("Failed to send message");
    } else {
        println!("Invalid index. Please provide a valid number.");
    }

    return sender_address;
}

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "127.0.0.8:12345"
        .parse()
        .expect("Failed to parse middleware address");
    let client_listener_address: SocketAddr = "127.0.0.8:8085"
        .parse()
        .expect("Failed to parse middleware address");
    let client_socket = UdpSocket::bind("127.0.0.8:3411")
        .await
        .expect("Failed to bind client socket");
    let client_socket_register = UdpSocket::bind("127.0.0.8:8090")
        .await
        .expect("Failed to bind client socket");
    let my_username = "Client1".to_string();
    register_user(client_socket_register, &my_username, "Client").await;
    //println!("Finished Registry");
    let middleware_socket = UdpSocket::bind(&middleware_address)
        .await
        .expect("Failed to bind middleware socket");
    let client_listener_socket = UdpSocket::bind(&client_listener_address)
        .await
        .expect("Failed to bind middleware socket");
    tokio::spawn(middleware_task(middleware_socket));
    tokio::spawn(client_listener(client_listener_socket));
    let termination = Arc::new(Mutex::new(0));
    let termination_clone = Arc::clone(&termination);

    let mut signal = signal(SignalKind::interrupt()).expect("Failed to create signal handler");
    tokio::spawn(async move {
        signal.recv().await;
        println!("Received termination signal");
        let server_addresses = ["127.0.0.2:21112", "127.0.0.3:21111", "127.0.0.4:21113"];

        let unregister_message = "UNREGISTER";
        let dos_socket = UdpSocket::bind("127.0.0.8:9000")
            .await
            .expect("Failed to bind socket");
        for server_address in &server_addresses {
            dos_socket
                .send_to(unregister_message.as_bytes(), server_address)
                .await
                .expect("Failed to send unregister message");
        }
        // Notify other tasks waiting for the signal
        //let _ = tx.send(());
        *termination_clone.lock().unwrap() = 1;

        // Exit the application
        std::process::exit(0);
        //return;
    });

    while *termination.lock().unwrap() == 0 {
        println!("Type a Number to undergo on of the following:");
        println!("1. View a Picture");
        println!("2. Encrypt an Image");
        println!("3. View Online Users");
        println!("4. Send an Image to a Client");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        if input.trim() == "2" {
            let mut client_buffer = [0; BUFFER_SIZE];
            loop {
                println!("Enter a number between 1-255 for views");
                let mut input = String::new();

                std::io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read line");

                match input.trim().parse::<u8>() {
                    Ok(parsed_num) => {
                        // Check if the parsed number is less than 255
                        if parsed_num < 255 && parsed_num > 0 {
                            unsafe { VIEWS = parsed_num };
                            break;
                        } else {
                            println!("Input is not in the range of 1-255: {}", parsed_num);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error parsing input as u8: {}", e);
                    }
                }
            }

            println!("Sending Image");
            let image_data = fs::read("image.png").expect("Failed to read the image file");
            let middleware_address = "127.0.0.8:12345"; // Replace with the actual middleware address and port
                                                        //sleep(Duration::from_millis(5000)).await;

            let packet_number = (image_data.len() / MAX_CHUNCK) + 1;
            for (index, piece) in image_data.chunks(MAX_CHUNCK).enumerate() {
                let is_last_piece = index == packet_number - 1;
                let chunk = Chunk {
                    views: unsafe { VIEWS },
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
                println!("Index:{}", index);
                let serialized = serde_json::to_string(&chunk).unwrap();

                client_socket
                    .send_to(&serialized.as_bytes(), middleware_address)
                    .await
                    .expect("Failed to send piece to middleware");

                let (num_bytes_received, _) = client_socket
                    .recv_from(&mut client_buffer)
                    .await
                    .expect("Failed to receive acknowledgement from server");
                let received_string = String::from_utf8_lossy(&client_buffer[..num_bytes_received]);
                println!("Received {}", received_string);
            }
            println!("Finished All Packets");
            println!("{}", packet_number);

            let mut encrypted_image_data: Vec<u8> = Vec::new();
            let mut image_chunks = HashMap::<i16, PacketArray>::new();
            let mut j = 0;
            let mut enecrypted_image_packet_number = packet_number;

            while j < enecrypted_image_packet_number {
                println!("j : {}  EIPN: {}", j, enecrypted_image_packet_number);
                if let Ok((_bytes_received, _client_address)) =
                    client_socket.recv_from(&mut client_buffer).await
                {
                    let packet_string = String::from_utf8_lossy(&client_buffer[0.._bytes_received]);
                    let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();
                    enecrypted_image_packet_number = deserialized.total_packet_number;
                    shift_left(&mut client_buffer, _bytes_received);

                    if j == enecrypted_image_packet_number - 1 {
                        image_chunks.insert(
                            enecrypted_image_packet_number.try_into().unwrap(),
                            deserialized.packet,
                        );
                        println!("Ana 5aragt {}", j);
                    } else {
                        image_chunks.insert(deserialized.position, deserialized.packet);
                    }
                    j += 1;
                    client_socket
                        .send_to("Ack".as_bytes(), _client_address)
                        .await
                        .expect("Failed to send");
                }
            }
            println!("Ana 5aragt");
            let image_chunks_cloned: BTreeMap<_, _> = image_chunks.clone().into_iter().collect();

            for (_key, value) in image_chunks_cloned {
                // println!("Key: {}, Value: {:?}", key, value);
                encrypted_image_data.extend_from_slice(&value);
            }

            remove_trailing_zeros(&mut encrypted_image_data);
            println!("EID Size {}", encrypted_image_data.len());

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

            println!("{}", decoded_image_data[0]);
        } else if input.trim() == "3" {
            let client_socket_query = UdpSocket::bind("127.0.0.8:8091")
                .await
                .expect("Failed to bind client socket");
            query_online_users(client_socket_query).await;
        } else if input.trim() == "1" {
            if let Ok(metadata) = fs::metadata("other_images/decoded.png") {
                if metadata.is_file() {
                    if let Err(err) = fs::remove_file("other_images/decoded.png") {
                        eprintln!("Error deleting file: {}", err);
                    } else {
                        println!("File deleted successfully");
                    }
                } else {
                    println!("The path exists, but it is not a file.");
                }
            } else {
                println!("The file does not exist.");
            }
            let png_files: Vec<_> = fs::read_dir("other_images/")
                .expect("Failed to read directory")
                .filter_map(|entry| {
                    entry.ok().and_then(|e| {
                        if e.path().extension().and_then(|e| e.to_str()) == Some("png") {
                            Some(e.file_name())
                        } else {
                            None
                        }
                    })
                })
                .collect();

            // Display enumerated PNG file names
            for (index, file_name) in png_files.iter().enumerate() {
                println!("{}: {}", index + 1, file_name.to_string_lossy());
            }

            // Ask the user to input a number
            println!("Type the number of the image you want to view:");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");

            // Parse the user input as an index
            let index: usize = input.trim().parse().expect("Invalid input");

            // Check if the index is valid
            if index > 0 && index <= png_files.len() {
                // Load and display the selected image
                let selected_file = &png_files[index - 1];
                let path = format!(
                    "other_images/{}",
                    selected_file.to_string_lossy().trim_matches('"')
                );
                let image_path = Path::new(&path);
                println!("{},{}", path, image_path.to_string_lossy());

                let encoded_image = file_as_image_buffer(format!(
                    "other_images/{}",
                    selected_file.to_string_lossy().trim_matches('"')
                ));
                let dec = Decoder::new(encoded_image);
                let out_buffer = dec.decode_alpha();
                let clean_buffer: Vec<u8> =
                    out_buffer.into_iter().filter(|b| *b != 0xff_u8).collect();
                let message = bytes_to_str(clean_buffer.as_slice());

                let mut decoded_image_data = base64::decode(message).unwrap_or_else(|e| {
                    eprintln!("Error decoding base64: {}", e);
                    Vec::new()
                });
                println!("{}", decoded_image_data[0]);
                if (decoded_image_data[0] == 0) {
                    println!("Image views have finished. Please send the client another request to increase them!");
                    //let file=selected_file.to_string_lossy();
                    //let parts: Vec<&str> = file.trim().split(',').collect();
                    //println!("Sender:{:?}",parts);
                    //let user_address: Vec<&str> =parts[1].trim().split(".p").collect();
                    //let sender_address=user_address[0].to_string();
                    //client_socket.send_to("VIEWS".as_bytes(), sender_address).await;
                    //let mut client_buffer = [0; BUFFER_SIZE];
                    //client_socket.recv_from(&mut client_buffer).await;
                    //println!("{}",client_buffer[7]);

                    if let Err(err) = fs::remove_file(format!(
                        "other_images/{}",
                        selected_file.to_string_lossy().trim_matches('"')
                    )) {
                        eprintln!("Error deleting file: {}", err);
                    } else {
                        println!("File deleted successfully");
                    }
                } if(decoded_image_data[0]>0) {
                    decoded_image_data[0] = decoded_image_data[0] - 1;

                    if let Ok(encrypted_image) = image::load_from_memory(&decoded_image_data[1..]) {
                        let (width, height) = encrypted_image.dimensions();
                        println!("Image dimensions: {} x {}", width, height);

                        if let Err(err) = encrypted_image.save("other_images/decoded.png") {
                            eprintln!("Failed to save the image: {}", err);
                        } else {
                            println!("Image saved successfully");
                        }
                    } else {
                        println!("Failed to create image from byte stream");
                    }

                    if let Ok(_image) = image::open("other_images/decoded.png") {
                        println!("Viewing image: decoded.png");
                        if let Ok(_) = open::that("other_images/decoded.png") {
                            println!("Opened Image");
                        } else {
                            println!("Failed to Open");
                        }
                    } else {
                        println!("Failed to open image");
                    }

                    let image_string = base64::encode(decoded_image_data.clone());
                    let payload = str_to_bytes(&image_string);
                    let destination_image =
                        file_as_dynamic_image(format!(
                            "other_images/{}",
                            selected_file.to_string_lossy().trim_matches('"')
                        ));
                    let enc = Encoder::new(payload, destination_image);
                    let result = enc.encode_alpha();
                    save_image_buffer(result, format!(
                        "other_images/{}",
                        selected_file.to_string_lossy().trim_matches('"')
                    ));
                }
            } else {
                println!("Invalid index. Please provide a valid number.");
            }
        } else if input.trim() == "4" {
            let client_socket_query = UdpSocket::bind("127.0.0.8:8091")
                .await
                .expect("Failed to bind client socket");
            let peer_address =
                query_online_users_to_send(client_socket_query, "Client1".to_string()).await;
            println!("{}", peer_address);
            let mut request_buffer = [0; BUFFER_SIZE];
            let mut small_buffer = [0; 8];
            client_socket
                .send_to("I want to request a picture".as_bytes(), peer_address)
                .await
                .expect("Failed to send");
            let (_, client_sender_address) =
                client_socket.recv_from(&mut small_buffer).await.expect("");
            let mut num_pictures = i32::from_be_bytes([
                small_buffer[4],
                small_buffer[5],
                small_buffer[6],
                small_buffer[7],
            ]);
            println!("Num Pics:{}", num_pictures);
            let mut i = 0;
            let mut enecrypted_image_packet_number: usize = 1000;
            let mut encrypted_image_data: Vec<u8> = Vec::new();
            let mut image_chunks = HashMap::<i16, PacketArray>::new();
            let mut j = 0;
            let mut image_vector: Vec<String> = Vec::new();
            while i < num_pictures {
                while j < enecrypted_image_packet_number {
                    println!("j : {}  EIPN: {}", j, enecrypted_image_packet_number);
                    if let Ok((_bytes_received, _client_address)) =
                        client_socket.recv_from(&mut request_buffer).await
                    {
                        let packet_string =
                            String::from_utf8_lossy(&request_buffer[0.._bytes_received]);
                        let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();
                        enecrypted_image_packet_number = deserialized.total_packet_number;
                        shift_left(&mut request_buffer, _bytes_received);
                        if j == enecrypted_image_packet_number - 1 {
                            image_chunks.insert(
                                enecrypted_image_packet_number.try_into().unwrap(),
                                deserialized.packet,
                            );
                            println!("Ana 5aragt {}", j);
                        } else {
                            image_chunks.insert(deserialized.position, deserialized.packet);
                        }
                        j += 1;
                        client_socket
                            .send_to("Ack".as_bytes(), _client_address)
                            .await
                            .expect("Failed to send");
                    }
                }

                j = 0;
                i += 1;

                let image_chunks_cloned: BTreeMap<_, _> =
                    image_chunks.clone().into_iter().collect();
                image_chunks.clear();

                for (_key, value) in image_chunks_cloned {
                    // println!("Key: {}, Value: {:?}", key, value);
                    encrypted_image_data.extend_from_slice(&value);
                }

                remove_trailing_zeros(&mut encrypted_image_data);
                println!("EID Size {}", encrypted_image_data.len());

                if let Ok(encrypted_image) = image::load_from_memory(&encrypted_image_data) {
                    let (width, height) = encrypted_image.dimensions();
                    println!("Image dimensions: {} x {}", width, height);

                    if let Err(err) =
                        encrypted_image.save(format!("other_images/encrypted{}.png", i))
                    {
                        eprintln!("Failed to save the image: {}", err);
                    } else {
                        println!("Image saved successfully");
                    }
                } else {
                    println!("Failed to create image from byte stream");
                }
                if let Ok(_image) = image::open(format!(
                    "other_images/encrypted{}.png",i
                )) {
                    if let Ok(_) = open::that(format!("other_images/encrypted{}.png", i)) {
                        println!("Opened Image");
                    } else {
                        println!("Failed to Open");
                    }
                } else {
                    println!("Failed to open image:");
                }
                image_vector.push(format!("other_images/encrypted{}.png", i));
                encrypted_image_data.clear();
            }
            let (num_bytes_received, client_address) = client_socket
                .recv_from(&mut request_buffer)
                .await
                .expect("Failed to receive data");

            // Deserialize the JSON into a vector of strings
            let png_files: Vec<String> =
                serde_json::from_slice(&request_buffer[..num_bytes_received])
                    .expect("Failed to deserialize JSON");

            // Display the list of strings to the user and let them choose
            println!("Choose a string:");
            for (index, png_file) in png_files.iter().enumerate() {
                println!("{}. {}", index + 1, png_file);
            }

            // Get user input for the chosen index
            let mut user_input = String::new();
            println!("Enter the number of the string you want to choose:");
            std::io::stdin()
                .read_line(&mut user_input)
                .expect("Failed to read user input");

            // Parse user input as usize
            let chosen_index: usize = user_input.trim().parse().expect("Invalid input");
            let mut chosen_string = "".to_string();

            // Ensure the chosen index is within bounds
            if chosen_index > 0 && chosen_index <= png_files.len() {
                // Retrieve the chosen string
                chosen_string = png_files[chosen_index - 1].to_string();
                println!("You chose: {}", chosen_string);
            } else {
                println!("Invalid choice");
            }
            for image_path in image_vector {
                if let Err(err) = fs::remove_file(image_path) {
                    eprintln!("Error deleting");
                }
            }
            client_socket
                .send_to(chosen_string.as_bytes(), client_address)
                .await
                .expect("Failed to send");
            let mut user_views = String::new();
            println!("Enter the number of views you want:");
            std::io::stdin()
                .read_line(&mut user_views)
                .expect("Failed to read user input");
            let chosen_views: usize = user_views.trim().parse().expect("Invalid input");
            let chosen_views_bytes: &[u8] = &chosen_views.to_be_bytes();
            client_socket
                .send_to(chosen_views_bytes, client_address)
                .await
                .expect("Failed to send");

            j = 0;
            while j < enecrypted_image_packet_number {
                if let Ok((_bytes_received, _client_address)) =
                    client_socket.recv_from(&mut request_buffer).await
                {
                    let packet_string =
                        String::from_utf8_lossy(&request_buffer[0.._bytes_received]);
                    let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();
                    enecrypted_image_packet_number = deserialized.total_packet_number;
                    shift_left(&mut request_buffer, _bytes_received);
                    if j == enecrypted_image_packet_number - 1 {
                        image_chunks.insert(
                            enecrypted_image_packet_number.try_into().unwrap(),
                            deserialized.packet,
                        );
                        println!("Ana 5aragt {}", j);
                    } else {
                        image_chunks.insert(deserialized.position, deserialized.packet);
                    }
                    j += 1;
                    client_socket
                        .send_to("Ack".as_bytes(), _client_address)
                        .await
                        .expect("Failed to send");
                }
            }
            let image_chunks_cloned: BTreeMap<_, _> = image_chunks.clone().into_iter().collect();

            for (_key, value) in image_chunks_cloned {
                // println!("Key: {}, Value: {:?}", key, value);
                encrypted_image_data.extend_from_slice(&value);
            }

            remove_trailing_zeros(&mut encrypted_image_data);
            println!("EID Size {}", encrypted_image_data.len());

            if let Ok(encrypted_image) = image::load_from_memory(&encrypted_image_data) {
                let (width, height) = encrypted_image.dimensions();
                println!("Image dimensions: {} x {}", width, height);
                let current_time = Local::now();
                let formatted_time = current_time.format("%Y%m%d%H%M%S").to_string();

                if let Err(err) = encrypted_image.save(format!(
                    "other_images/encrypted{},{}.png",
                    formatted_time, client_sender_address
                )) {
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
