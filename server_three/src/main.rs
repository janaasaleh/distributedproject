use async_std::net::UdpSocket;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::net::SocketAddr;
use std::time::Duration;
use steganography::encoder::*;
use steganography::util::*;

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
#[derive(Serialize, Deserialize, Debug)]
struct User {
    address: String,
    name: String,
    user_type: String,
}

fn extract_ip_address(socket_addr: SocketAddr) -> String {
    if let Some(ip) = socket_addr.ip().to_string().split(':').next() {
        return ip.to_string();
    }
    String::new()
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
        let packet_string = String::from_utf8_lossy(&buffer[0.._bytes_received]);
        let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();
        shift_left(&mut buffer, _bytes_received);

        if deserialized.position != -1 {
            image_chunks.insert(deserialized.position, deserialized.packet);
            packet_number += 1;
            socket
                .send_to(
                    "Sent acknowledgement to middleware".as_bytes(),
                    _client_address,
                )
                .await
                .expect("Couldnt send to middleware");
        } else {
            println!("Entered in Server Encryption");
            let mut real_client_address = "".to_string();
            if let Ok((client_address_bytes_received, client_address)) =
                socket.recv_from(&mut buffer).await
            {
                real_client_address =
                    String::from_utf8_lossy(&buffer[..client_address_bytes_received]).to_string();
            }

            image_chunks.insert(packet_number, deserialized.packet);
            packet_number = 1;

            let image_chunks_cloned: BTreeMap<_, _> = image_chunks.clone().into_iter().collect();

            for (_key, value) in image_chunks_cloned {
                // println!("Key: {}, Value: {:?}", key, value);
                image_data.extend_from_slice(&value);
            }

            remove_trailing_zeros(&mut image_data);

            let image_string = base64::encode(image_data.clone());
            let payload = str_to_bytes(&image_string);
            let destination_image = file_as_dynamic_image("encrypt.png".to_string());
            let enc = Encoder::new(payload, destination_image);
            let result = enc.encode_alpha();
            save_image_buffer(result, "encrypted.png".to_string());

            println!("Sending encrypted image");
            image_data.clear();
            image_data = fs::read("encrypted.png").expect("Failed to read the image file");

            let packet_number = (image_data.len() / MAX_CHUNCK) + 1;
            println!("{}", packet_number);

            for (index, piece) in image_data.chunks(MAX_CHUNCK).enumerate() {
                let is_last_piece = index == packet_number - 1;
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

                socket
                    .send_to(&serialized.as_bytes(), &real_client_address)
                    .await
                    .expect("Failed to send piece to middleware");
                println!("Server sent packet {}", index);
                socket
                    .recv_from(&mut buffer)
                    .await
                    .expect("Failed to receive acknowledgement from server");
                println!("Server received ack packet {}", index);
            }
            image_data.clear();
        }
    }
}

async fn server_middleware(middleware_address: &str, server_addresses: Vec<&str>) {
    let middleware_socket = UdpSocket::bind(middleware_address)
        .await
        .expect("Failed to bind middleware socket");
    let server_to_server_socket = UdpSocket::bind("127.0.0.4:8080")
        .await
        .expect("Failed to bind server to server socket");
    let server_load_socket = UdpSocket::bind("127.0.0.4:8100")
        .await
        .expect("Failed to bind server to server socket");

    let mut server_down = 0;
    let real_server_down = 0;
    let server_down_index: i32 = 10;
    let server_up_index: i32 = 20;
    let mut own_down: i32 = 0;
    let mut which_server: i32 = 500;
    let mut just_up: i32 = 0;
    let mut previous_down: i32 = 0;
    let mut just_slept: i32 = 0;
    let mut server_down_requests: i32 = 0;
    let mut num_down: i32 = 0;
    let max_usage: i32 = 1000000;
    println!("Server middleware is listening on {}", middleware_address);
    let mut current_server: i32 = 0;
    let mut receive_buffer = [0; BUFFER_SIZE];
    let mut send_buffer = [0; BUFFER_SIZE]; // Separate buffer for sending data
    let mut my_load = String::from(""); //New
    let mut current_packet = 0; //New
    let mut _my_packets = 2000; //New
    let mut active_users: Vec<User> = Vec::new();

    while let Ok((bytes_received, client_address)) =
        middleware_socket.recv_from(&mut receive_buffer).await
    {
        let message = String::from_utf8_lossy(&receive_buffer[..bytes_received]);

        if message.starts_with("REGISTER:") {
            // Extract username and user type from message
            let parts: Vec<&str> = message.trim().split(':').collect();
            let username = parts[1].trim();
            let user_type = parts[2].trim();
            let user_address: SocketAddr = client_address;
            let user_s_address = extract_ip_address(user_address);
            let new_user = User {
                address: user_s_address,
                name: username.to_string(),
                user_type: user_type.to_string(),
            };
            active_users.push(new_user);
            println!("User registered: {} ({})", username, user_type);
            let response = "User registered";
            middleware_socket
                .send_to(response.as_bytes(), client_address)
                .await
                .expect("Failed to send response");
            continue;
        } else if message.starts_with("UNREGISTER") {
            // Extract user's address and unregister
            let user_address: SocketAddr = client_address;
            let user_s_address = extract_ip_address(user_address);
            if let Some(index) = active_users
                .iter()
                .position(|user| user.address == user_s_address)
            {
                active_users.remove(index);
                println!("User unregistered: {}", user_s_address);
            }
            let response = "User unregistered";
            middleware_socket
                .send_to(response.as_bytes(), client_address)
                .await
                .expect("Failed to send response");
            continue;
        }
        let ip = client_address.ip(); //New
        let ip_string = ip.to_string(); // New
        let mut rng = rand::thread_rng();
        let cpu_usage: i32 = rng.gen_range(0..10000);
        let real_cpu_usage: &[u8] = &cpu_usage.to_be_bytes();
        println!("My Load {}", my_load);
        println!("Current Server: {}", current_server);
        println!("Entered Here 1");

        let mut rng = rand::thread_rng();
        let random_number: u32 = rng.gen_range(0..10);
        let index: &[u8] = &current_server.to_be_bytes();
        let down_index: &[u8] = &server_down_index.to_be_bytes();
        let up_index: &[u8] = &server_up_index.to_be_bytes();
        //let mut otherload1;
        //let mut otherload2;

        let mut server_to_server1_receive_buffer = [0; 4];
        let mut server_to_server2_receive_buffer = [0; 4];
        let mut just_up_receive_buffer = [0; 4];
        let mut server1_load_receive_buffer = [0; BUFFER_SIZE];
        let mut server2_load_receive_buffer = [0; BUFFER_SIZE];

        if my_load != ip_string {
            if my_load != "" {
                let temp_cpu_usage: &[u8] = &max_usage.to_be_bytes();
                server_to_server_socket
                    .send_to(temp_cpu_usage, "127.0.0.3:8080")
                    .await
                    .expect("Failed to send index to server 2");
                server_to_server_socket
                    .send_to(temp_cpu_usage, "127.0.0.2:8080")
                    .await
                    .expect("Failed to send index to server 3");
            } else {
                server_to_server_socket
                    .send_to(real_cpu_usage, "127.0.0.3:8080")
                    .await
                    .expect("Failed to send index to server 2");
                server_to_server_socket
                    .send_to(real_cpu_usage, "127.0.0.2:8080")
                    .await
                    .expect("Failed to send index to server 3");
            }
        }

        if (my_load != ip_string) {
            server_to_server_socket
                .recv_from(&mut server_to_server1_receive_buffer)
                .await
                .expect("Couldn't recieve index");

            server_to_server_socket
                .recv_from(&mut server_to_server2_receive_buffer)
                .await
                .expect("Couldn't recieve index");
        }
        let mut index1 = i32::from_be_bytes([
            server_to_server1_receive_buffer[0],
            server_to_server1_receive_buffer[1],
            server_to_server1_receive_buffer[2],
            server_to_server1_receive_buffer[3],
        ]);
        let mut index2 = i32::from_be_bytes([
            server_to_server2_receive_buffer[0],
            server_to_server2_receive_buffer[1],
            server_to_server2_receive_buffer[2],
            server_to_server2_receive_buffer[3],
        ]);

        println!("Index recieved {}", index1);
        println!("Index recieved {}", index2);

        if (my_load != ip_string) {
            if message == "QUERY" && cpu_usage < index1 && cpu_usage < index2 {
                // Send the list of active users to the requesting client
                let users_list: String = active_users
                    .iter()
                    .map(|user| user.name.clone())
                    .collect::<Vec<String>>()
                    .join(",");
                middleware_socket
                    .send_to(users_list.as_bytes(), client_address)
                    .await
                    .expect("Failed to send active users list");
                continue;
            } else if message == "QUERY2" && cpu_usage < index1 && cpu_usage < index2 {
                // Send the list of active users to the requesting client
                let active_users_json = serde_json::to_string(&active_users)
                    .expect("Failed to serialize active users to JSON");
                middleware_socket
                    .send_to(active_users_json.as_bytes(), client_address)
                    .await
                    .expect("Failed to send active users list");
                continue;
            } else if (cpu_usage < index1 && cpu_usage < index2 && my_load == "") {
                my_load = ip_string;
                let packet_string = String::from_utf8_lossy(&receive_buffer[0..bytes_received]);
                let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();
                _my_packets = deserialized.total_packet_number;
            } else {
                continue;
            }
        }

        let server_index = 2; // You can implement load balancing logic here
        let server_address = server_addresses[server_index];
        let server_address: SocketAddr = server_address
            .parse()
            .expect("Failed to parse server address");

        let server_socket = UdpSocket::bind("127.0.0.4:0")
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

        //let (ack_bytes_received, server_caddress) = server_socket
        //            .recv_from(&mut receive_buffer)
        //            .await
        //            .expect("Failed to receive acknowledgment from server");
        //
        //middleware_socket
        //.send_to(&receive_buffer[0..bytes_received], client_address)
        //.await
        //.expect("Failed to send data to server");

        println!("Entered Here 2");
        println!("{}", current_packet);
        let packet_string = String::from_utf8_lossy(&receive_buffer[0..bytes_received]);
        let deserializeds: Chunk = serde_json::from_str(&packet_string).unwrap();
        println!("{}", deserializeds.position);
        current_packet += 1;
        if current_packet == _my_packets {
            println!("Entered MAX Packet size");
            let mut i = 0;
            let mut encrypted_image_packets = _my_packets;
            middleware_socket
                .send_to("".as_bytes(), client_address)
                .await
                .expect("Failed to send acknowledgment to client");
            middleware_socket
                .send_to(client_address.to_string().as_bytes(), server_address)
                .await
                .expect("Failed to send acknowledgment to client");
            println!("Just chill bara");
            current_packet = 0;
            my_load = "".to_string();
        } else {
            let (ack_bytes_received, server_caddress) = server_socket
                .recv_from(&mut receive_buffer)
                .await
                .expect("Failed to receive acknowledgment from server");
            middleware_socket
                .send_to(&receive_buffer[..ack_bytes_received], client_address)
                .await
                .expect("Failed to send acknowledgment to client");

            // Clear the receive buffer for the next request
            server_to_server1_receive_buffer = [0; 4];
            server_to_server2_receive_buffer = [0; 4];
            receive_buffer = [0; BUFFER_SIZE];
        }
    }
}

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "127.0.0.4:21113"
        .parse()
        .expect("Failed to parse middleware address");
    let middleware_address_str = middleware_address.to_string();

    // Define the server addresses and middleware addresses
    let server_addresses = ["127.0.0.2:54321", "127.0.0.3:54322", "127.0.0.4:54323"];
    let server1_task = server1("127.0.0.4:54323", &middleware_address_str);

    // Start the server middleware
    let server_middleware_task =
        server_middleware(&middleware_address_str, server_addresses.to_vec());

    let _ = tokio::join!(server1_task, server_middleware_task);
}
