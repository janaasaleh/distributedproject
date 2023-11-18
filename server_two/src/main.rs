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

async fn server3(server_address: &str, _middleware_address: &str) {
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
                    .send_to(&serialized.as_bytes(), _client_address)
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
    let server_to_server_socket = UdpSocket::bind("127.0.0.3:8080")
        .await
        .expect("Failed to bind server to server socket");
    let just_up_socket = UdpSocket::bind("127.0.0.3:8087")
        .await
        .expect("Failed to bind server to server socket");
    let skip_socket = UdpSocket::bind("127.0.0.3:8088")
        .await
        .expect("Failed to bind server to server socket");
    let server_load_socket = UdpSocket::bind("127.0.0.3:8100")
        .await
        .expect("Failed to bind server to server socket");

    let mut server_down = 0;
    let mut real_server_down = 0;
    let mut server_down_index: i32 = 11;
    let mut server_up_index: i32 = 21;
    let mut own_down: i32 = 0;
    let mut which_server: i32 = 500;
    let mut just_up: i32 = 0;
    let mut previous_down: i32 = 0;
    let mut just_slept: i32 = 0;
    let mut server_down_requests: i32 = 0;
    let mut num_down: i32 = 0;
    println!("Server middleware is listening on {}", middleware_address);
    let mut current_server: i32 = 0;
    let mut receive_buffer = [0; BUFFER_SIZE];
    let mut send_buffer = [0; BUFFER_SIZE]; // Separate buffer for sending data
    let mut my_load = String::from(""); //New
    let mut current_packet = 0; //New
    let mut _my_packets = 2000; //New
    while let Ok((bytes_received, client_address)) =
        middleware_socket.recv_from(&mut receive_buffer).await
    {
        let ip = client_address.ip(); //New
        let ip_string = ip.to_string(); // New

        println!("Just Slept:{}", just_slept);
        if (just_slept == 1) {
            just_slept = 0;
            continue;
        } else if just_slept > 1 {
            just_slept = just_slept - 1;
            continue;
        } else {
            just_slept = 0;
        }
        if (which_server == 2) {
            server_down_requests += 1;
        }
        if (which_server == 500) {
            server_down_requests = 0;
        }
        println!("Current Server: {}", current_server);
        println!("Entered Here 1");
        println!("SDR:{}", server_down_requests);

        let mut rng = rand::thread_rng();
        let random_number: u32 = rng.gen_range(0..10);
        let index: &[u8] = &current_server.to_be_bytes();
        let down_index: &[u8] = &server_down_index.to_be_bytes();
        let up_index: &[u8] = &server_up_index.to_be_bytes();

        let mut server_to_server1_receive_buffer = [0; 4];
        let mut server_to_server2_receive_buffer = [0; 4];
        let mut just_up_receive_buffer = [0; 4];
        let mut server1_load_receive_buffer = [0; BUFFER_SIZE];
        let mut server2_load_receive_buffer = [0; BUFFER_SIZE];

        //server_to_server_socket
        //.connect("127.0.0.2:8080")
        //.await
        //.expect("Failed to connect to the server");
        //
        //server_to_server_socket
        //.connect("127.0.0.4:8080")
        //.await
        //.expect("Failed to connect to the server");
        if random_number < 4
            && server_down == 0
            && real_server_down == 0
            && previous_down == 0
            && current_server != 1
            && my_load == ""
        {
            server_down = 0;
            own_down = 1;
            server_to_server_socket
                .send_to(down_index, "127.0.0.2:8080")
                .await
                .expect("Failed to send index to server 1");
            server_to_server_socket
                .send_to(down_index, "127.0.0.4:8080")
                .await
                .expect("Failed to send index to server 3");
            server_load_socket
                .send_to(my_load.as_bytes(), "127.0.0.2:8100")
                .await
                .expect("Failed to send index to server 3");
            server_load_socket
                .send_to(my_load.as_bytes(), "127.0.0.4:8100")
                .await
                .expect("Failed to send index to server 3");
            println!("Server 2 going down!");
        } else {
            if (which_server == 500) {
                server_to_server_socket
                    .send_to(index, "127.0.0.2:8080")
                    .await
                    .expect("Failed to send index to server 1");
                server_to_server_socket
                    .send_to(index, "127.0.0.4:8080")
                    .await
                    .expect("Failed to send index to server 3");
                server_load_socket
                    .send_to(my_load.as_bytes(), "127.0.0.2:8100")
                    .await
                    .expect("Failed to send index to server 3");
                server_load_socket
                    .send_to(my_load.as_bytes(), "127.0.0.4:8100")
                    .await
                    .expect("Failed to send index to server 3");
            } else if which_server == 0 {
                server_to_server_socket
                    .send_to(index, "127.0.0.4:8080")
                    .await
                    .expect("Failed to send index to server 3");
                server_load_socket
                    .send_to(my_load.as_bytes(), "127.0.0.4:8100")
                    .await
                    .expect("Failed to send index to server 3");
            } else if which_server == 2 {
                server_to_server_socket
                    .send_to(index, "127.0.0.2:8080")
                    .await
                    .expect("Failed to send index to server 1");
                println!("Ba3atelo");

                server_load_socket
                    .send_to(my_load.as_bytes(), "127.0.0.2:8100")
                    .await
                    .expect("Failed to send index to server 3");
            }
        }

        let (ll_bytes, _) = server_load_socket
            .recv_from(&mut server1_load_receive_buffer)
            .await
            .expect("Couldn't recieve index");
        server_to_server_socket
            .recv_from(&mut server_to_server1_receive_buffer)
            .await
            .expect("Couldn't recieve index");
        let mut index1 = i32::from_be_bytes([
            server_to_server1_receive_buffer[0],
            server_to_server1_receive_buffer[1],
            server_to_server1_receive_buffer[2],
            server_to_server1_receive_buffer[3],
        ]);

        let mut otherload1 =
            String::from_utf8_lossy(&server1_load_receive_buffer[0..ll_bytes]).to_string();
        let mut ss_bytes = 0;

        if (server_down == 0 || index1 > 19) {
            server_to_server_socket
                .recv_from(&mut server_to_server2_receive_buffer)
                .await
                .expect("Couldn't recieve index");
            let (ss_rrr, _) = server_load_socket
                .recv_from(&mut server2_load_receive_buffer)
                .await
                .expect("Couldn't recieve index");
            ss_bytes = ss_rrr;
        }

        let mut index2 = i32::from_be_bytes([
            server_to_server2_receive_buffer[0],
            server_to_server2_receive_buffer[1],
            server_to_server2_receive_buffer[2],
            server_to_server2_receive_buffer[3],
        ]);

        let mut otherload2 =
            String::from_utf8_lossy(&server2_load_receive_buffer[0..ss_bytes]).to_string();

        println!("otherload2 hex: {:?}", otherload2.as_bytes());
        println!("otherload1 hex: {:?}", otherload1.as_bytes());
        println!("otherload1 hex: {:?}", ip_string.as_bytes());

        println!("Index recieved {}", index1);
        println!("Index recieved {}", index2);

        if (index1 > 9 && index1 < 20 && index2 > 9 && index2 < 20 && own_down == 1) {
            index1 = current_server;
            index2 = current_server;
            own_down = 0;
            println!("All 3 servers requested sleep. Overturned!")
        } else if (own_down == 1 && index1 > 9 && index1 < 20) {
            own_down = 0;
            index1 = current_server;
            println!("2 servers requested sleep. Overturned!")
        } else if (own_down == 1 && index2 > 9 && index2 < 20) {
            own_down = 0;
            index2 = current_server;
            println!("2 servers requested sleep. Overturned!")
        } else if (index1 > 9 && index1 < 20 && index2 > 9 && index2 < 20 && own_down == 0) {
            index1 = current_server;
            index2 = current_server;
            println!("2 servers not me requested sleep. Overturned!")
        }

        if (index1 > 9) {
            if index1 < 19 {
                if (index1 == 10) {
                    println!("Server 1 is down");
                    server_down = 1;
                    which_server = 0;
                }
                if (index1 == 12) {
                    println!("Server 3 is down");
                    server_down = 1;
                    which_server = 2;
                }
            } else {
                if index1 == 20 {
                    println!("Server 1 is back up");
                    server_down = 0;
                    which_server = 500;
                }
                if index1 == 22 {
                    println!("Server 3 is back up");
                    server_down = 0;
                    which_server = 500;
                    just_up = 1;
                }
            }
        }
        if (index2 > 9) {
            if index2 < 19 {
                if (index2 == 10) {
                    println!("Server 1 is down");
                    server_down = 1;
                    which_server = 0;
                }
                if (index2 == 12) {
                    println!("Server 3 is down");
                    server_down = 1;
                    which_server = 2;
                }
            } else {
                if index2 == 20 {
                    println!("Server 1 is back up");
                    server_down = 0;
                    which_server = 500;
                }
                if index2 == 22 {
                    println!("Server 3 is back up");
                    server_down = 0;
                    which_server = 500;
                    just_up = 1;
                }
            }
        }

        if (which_server == 500) {
            if current_server == 0 {
                current_server += 1;
                if (just_up == 1) {
                    let cs: &[u8] = &current_server.to_be_bytes();
                    server_to_server_socket
                        .send_to(cs, "127.0.0.4:8087")
                        .await
                        .expect("Failed to send index to server 3");
                    let sdr: &[u8] = &server_down_requests.to_be_bytes();
                    server_to_server_socket
                        .send_to(sdr, "127.0.0.4:8088")
                        .await
                        .expect("Failed to send index to server 3");

                    //server_to_server_socket
                    //.send_to(cs, "127.0.0.4:8080")
                    //.await.expect("Failed to send index to server 3");
                    just_up = 0;
                }
                if (own_down == 1) {
                    tokio::time::sleep(Duration::from_secs(40)).await;
                    previous_down = 6;
                    println!("Server 2 going up!");
                    server_to_server_socket
                        .send_to(up_index, "127.0.0.2:8080")
                        .await
                        .expect("Failed to send index to server 1");
                    server_to_server_socket
                        .send_to(up_index, "127.0.0.4:8080")
                        .await
                        .expect("Failed to send index to server 2");
                    server_load_socket
                        .send_to("".as_bytes(), "127.0.0.2:8100")
                        .await
                        .expect("Failed to send index to server 1");
                    server_load_socket
                        .send_to("".as_bytes(), "127.0.0.4:8100")
                        .await
                        .expect("Failed to send index to server 2");
                    own_down = 0;
                    server_down = 0;
                    just_up_socket
                        .recv_from(&mut just_up_receive_buffer)
                        .await
                        .expect("Couldn't recieve index");
                    current_server = i32::from_be_bytes([
                        just_up_receive_buffer[0],
                        just_up_receive_buffer[1],
                        just_up_receive_buffer[2],
                        just_up_receive_buffer[3],
                    ]);
                    just_up_receive_buffer = [0; 4];
                    skip_socket
                        .recv_from(&mut just_up_receive_buffer)
                        .await
                        .expect("Couldn't recieve index");
                    just_slept = i32::from_be_bytes([
                        just_up_receive_buffer[0],
                        just_up_receive_buffer[1],
                        just_up_receive_buffer[2],
                        just_up_receive_buffer[3],
                    ]);
                    //current_server=1-current_server;
                    if (just_slept > 90) {
                        just_slept = 4;
                    }
                    println!("Current Server Down {}", current_server);
                    println!("I am here");
                    just_up_receive_buffer = [0; 4];
                    //receive_buffer = [0; BUFFER_SIZE];
                    //receive_buffer = [0; BUFFER_SIZE];
                    num_down += 1;
                    if (num_down > 1) {
                        //just_slept=just_slept+1;
                        just_slept = just_slept;
                    }
                }
                if ip_string == my_load {
                } else {
                    continue;
                }
            } else if current_server == 1 {
                if (just_up == 1) {
                    current_server += 1;
                    let cs: &[u8] = &current_server.to_be_bytes();
                    server_to_server_socket
                        .send_to(cs, "127.0.0.4:8087")
                        .await
                        .expect("Failed to send index to server 3");
                    let sdr: &[u8] = &server_down_requests.to_be_bytes();
                    server_to_server_socket
                        .send_to(sdr, "127.0.0.4:8088")
                        .await
                        .expect("Failed to send index to server 3");
                    //server_to_server_socket
                    //.send_to(cs, "127.0.0.4:8080")
                    //.await.expect("Failed to send index to server 3");
                    just_up = 0;
                } else {
                    current_server += 1;
                }

                if ip_string == my_load {
                    my_load = my_load;
                } else if ((my_load == "")
                    && !(otherload1 == ip_string)
                    && !(otherload2 == ip_string))
                {
                    println!("IP:{}", ip_string);
                    println!("{},{}", otherload1, otherload2);
                    println!("Da5alt hena ezay");
                    if (ip_string != otherload2 && ip_string != otherload1) {
                        println!("Not Equal");
                    } else {
                        println!("Equal");
                    }
                    println!("my_load: {}", my_load);
                    println!("otherload1: {}", otherload1 == ip_string);
                    println!("otherload2: {}", otherload2 == ip_string);
                    println!("ip_string: |{}|", ip_string);
                    println!("otherload2 hex: {:?}", otherload2.as_bytes());
                    println!("otherload1 hex: {:?}", otherload1.as_bytes());
                    println!("ip_string hex: {:?}", ip_string.as_bytes());
                    println!("otherload1: |{}|", otherload1);
                    println!("otherload2: |{}|", otherload2);

                    let ip_strings: String = ip.to_string();
                    my_load = ip_strings;
                    let packet_string = String::from_utf8_lossy(&receive_buffer[0..bytes_received]);
                    let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();

                    _my_packets = deserialized.total_packet_number;
                    println!("{}", _my_packets);
                    //packets_size_socket
                    //    .recv_from(&mut packets_size_receive_buffer)
                    //    .await
                    //    .expect("Couldn't recieve index");
                    // _my_packets = i32::from_be_bytes([
                    //packets_size_receive_buffer[0],
                    //packets_size_receive_buffer[1],
                    //packets_size_receive_buffer[2],
                    //packets_size_receive_buffer[3],
                    //]);
                } else {
                    continue;
                }
            } else if current_server == 2 {
                current_server = 0;
                if (just_up == 1) {
                    let cs: &[u8] = &current_server.to_be_bytes();
                    server_to_server_socket
                        .send_to(cs, "127.0.0.4:8087")
                        .await
                        .expect("Failed to send index to server 3");
                    let sdr: &[u8] = &server_down_requests.to_be_bytes();
                    server_to_server_socket
                        .send_to(sdr, "127.0.0.4:8088")
                        .await
                        .expect("Failed to send index to server 3");
                    //server_to_server_socket
                    //.send_to(cs, "127.0.0.4:8080")
                    //.await.expect("Failed to send index to server 3");
                    just_up = 0;
                }
                if (own_down == 1) {
                    tokio::time::sleep(Duration::from_secs(40)).await;
                    previous_down = 6;
                    println!("Server 2 going up!");
                    server_to_server_socket
                        .send_to(up_index, "127.0.0.2:8080")
                        .await
                        .expect("Failed to send index to server 1");
                    server_to_server_socket
                        .send_to(up_index, "127.0.0.4:8080")
                        .await
                        .expect("Failed to send index to server 2");
                    server_load_socket
                        .send_to("".as_bytes(), "127.0.0.2:8100")
                        .await
                        .expect("Failed to send index to server 1");
                    server_load_socket
                        .send_to("".as_bytes(), "127.0.0.4:8100")
                        .await
                        .expect("Failed to send index to server 2");
                    own_down = 0;
                    server_down = 0;
                    just_up_socket
                        .recv_from(&mut just_up_receive_buffer)
                        .await
                        .expect("Couldn't recieve index");
                    current_server = i32::from_be_bytes([
                        just_up_receive_buffer[0],
                        just_up_receive_buffer[1],
                        just_up_receive_buffer[2],
                        just_up_receive_buffer[3],
                    ]);
                    just_up_receive_buffer = [0; 4];
                    skip_socket
                        .recv_from(&mut just_up_receive_buffer)
                        .await
                        .expect("Couldn't recieve index");
                    just_slept = i32::from_be_bytes([
                        just_up_receive_buffer[0],
                        just_up_receive_buffer[1],
                        just_up_receive_buffer[2],
                        just_up_receive_buffer[3],
                    ]);
                    //current_server=1-current_server;
                    if (just_slept > 90) {
                        just_slept = 4;
                    }
                    println!("Current Server Down {}", current_server);
                    println!("I am here");
                    just_up_receive_buffer = [0; 4];
                    //receive_buffer = [0; BUFFER_SIZE];
                    //receive_buffer = [0; BUFFER_SIZE];
                    num_down += 1;
                    if (num_down > 1) {
                        //just_slept=just_slept+1;
                        just_slept = just_slept;
                    }
                }
                if ip_string == my_load {
                } else {
                    continue;
                }
            }
        } else if (which_server == 0) {
            if current_server == 0 && which_server == 0 {
                current_server += 1;
                if ip_string == my_load {
                } else {
                    continue;
                }
            } else if current_server == 1 && which_server == 0 {
                current_server += 1;
                if ip_string == my_load {
                } else if ((my_load == "")
                    && !(otherload1 == ip_string)
                    && !(otherload2 == ip_string))
                {
                    println!("IP:{}", ip_string);
                    println!("{},{}", otherload1, otherload2);
                    println!("Da5alt hena ezay");
                    if (ip_string != otherload2 && ip_string != otherload1) {
                        println!("Not Equal");
                    } else {
                        println!("Equal");
                    }
                    println!("my_load: {}", my_load);
                    println!("otherload1: {}", otherload1 == ip_string);
                    println!("otherload2: {}", otherload2 == ip_string);
                    println!("ip_string: |{}|", ip_string);
                    println!("otherload2 hex: {:?}", otherload2.as_bytes());
                    println!("otherload1 hex: {:?}", otherload1.as_bytes());
                    println!("ip_string hex: {:?}", ip_string.as_bytes());
                    println!("otherload1: |{}|", otherload1);
                    println!("otherload2: |{}|", otherload2);

                    let ip_strings: String = ip.to_string();
                    my_load = ip_strings;
                    let packet_string = String::from_utf8_lossy(&receive_buffer[0..bytes_received]);
                    let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();

                    _my_packets = deserialized.total_packet_number;
                    println!("{}", _my_packets);
                    //packets_size_socket
                    //    .recv_from(&mut packets_size_receive_buffer)
                    //    .await
                    //    .expect("Couldn't recieve index");
                    // _my_packets = i32::from_be_bytes([
                    //packets_size_receive_buffer[0],
                    //packets_size_receive_buffer[1],
                    //packets_size_receive_buffer[2],
                    //packets_size_receive_buffer[3],
                    //]);
                } else {
                    continue;
                }
            } else if current_server == 2 && which_server == 0 {
                current_server = 1;
                if ip_string == my_load {
                } else {
                    continue;
                }
            }
        } else if (which_server == 2) {
            if current_server == 0 {
                current_server += 1;
                if ip_string == my_load {
                } else {
                    continue;
                }
            } else if current_server == 1 {
                current_server = 0;
                if ip_string == my_load {
                } else if ((my_load == "")
                    && !(otherload1 == ip_string)
                    && !(otherload2 == ip_string))
                {
                    println!("IP:{}", ip_string);
                    println!("{},{}", otherload1, otherload2);
                    println!("Da5alt hena ezay");
                    if (ip_string != otherload2 && ip_string != otherload1) {
                        println!("Not Equal");
                    } else {
                        println!("Equal");
                    }
                    println!("my_load: {}", my_load);
                    println!("otherload1: {}", otherload1 == ip_string);
                    println!("otherload2: {}", otherload2 == ip_string);
                    println!("ip_string: |{}|", ip_string);
                    println!("otherload2 hex: {:?}", otherload2.as_bytes());
                    println!("otherload1 hex: {:?}", otherload1.as_bytes());
                    println!("ip_string hex: {:?}", ip_string.as_bytes());
                    println!("otherload1: |{}|", otherload1);
                    println!("otherload2: |{}|", otherload2);

                    let ip_strings: String = ip.to_string();
                    my_load = ip_strings;
                    let packet_string = String::from_utf8_lossy(&receive_buffer[0..bytes_received]);
                    let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();

                    _my_packets = deserialized.total_packet_number;
                    println!("{}", _my_packets);
                    //packets_size_socket
                    //    .recv_from(&mut packets_size_receive_buffer)
                    //    .await
                    //    .expect("Couldn't recieve index");
                    // _my_packets = i32::from_be_bytes([
                    //packets_size_receive_buffer[0],
                    //packets_size_receive_buffer[1],
                    //packets_size_receive_buffer[2],
                    //packets_size_receive_buffer[3],
                    //]);
                } else {
                    continue;
                }
            } else if current_server == 2 {
                current_server = 0;
                if ip_string == my_load {
                } else {
                    continue;
                }
            }
        } else {
            println!("No which server variable");
        }

        let server_index = 1; // You can implement load balancing logic here
        let server_address = server_addresses[server_index];
        let server_address: SocketAddr = server_address
            .parse()
            .expect("Failed to parse server address");

        let server_socket = UdpSocket::bind("127.0.0.3:0")
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

        //let (ack_bytes_received, server_caddress) = server_socket
        //    .recv_from(&mut receive_buffer)
        //    .await
        //    .expect("Failed to receive acknowledgment from server");
        //println!("Entered Here 3");
        //println!("Server address {}", server_caddress);
        //
        //// Send the acknowledgment from the server to the client's middleware
        //middleware_socket
        //    .send_to(&receive_buffer[..ack_bytes_received], client_address)
        //    .await
        //    .expect("Failed to send acknowledgment to client");
        //println!("Entered Here 4");
        //println!("Client Address:{}",client_address);

        // Clear the receive buffer for the next request
        println!("{}", current_packet);
        current_packet += 1;
        if current_packet == _my_packets {
            println!("Entered MAX Packet size");
            let mut i = 0;
            let mut encrypted_image_packets = _my_packets;
            middleware_socket
                .send_to("".as_bytes(), client_address)
                .await
                .expect("Failed to send acknowledgment to client");
            while i < encrypted_image_packets {
                println!("Just chill");
                let (ack_bytes_received, server_caddress) = server_socket
                    .recv_from(&mut receive_buffer)
                    .await
                    .expect("Failed to receive acknowledgment from server");
                let packet_string = String::from_utf8_lossy(&receive_buffer[0..ack_bytes_received]);
                let deserialized: Chunk = serde_json::from_str(&packet_string).unwrap();
                encrypted_image_packets = deserialized.total_packet_number;
                println!("Entered Here 3");
                //println!("Server address {}", server_caddress);

                // Send the acknowledgment from the server to the client's middleware
                middleware_socket
                    .send_to(&receive_buffer[0..ack_bytes_received], client_address)
                    .await
                    .expect("Failed to send acknowledgment to client");
                shift_left(&mut receive_buffer, ack_bytes_received);

                middleware_socket
                    .recv_from(&mut receive_buffer)
                    .await
                    .expect("Failed to send acknowledgment to client");

                let ack: String = "Ack".to_string();
                server_socket
                    .send_to(&ack.as_bytes(), server_caddress)
                    .await
                    .expect("Failed to send acknowledgment to client");
                println!("Entered Here 4");
                println!("Client Address:{}", client_address);
                i += 1;
                println!("Index {}", i);
            }
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
            server_to_server1_receive_buffer = [0; 4];
            server_to_server2_receive_buffer = [0; 4];
            receive_buffer = [0; BUFFER_SIZE];
            if (own_down == 1) {
                tokio::time::sleep(Duration::from_secs(40)).await;
                previous_down = 6;
                println!("Server 2 going up!");
                server_to_server_socket
                    .send_to(up_index, "127.0.0.2:8080")
                    .await
                    .expect("Failed to send index to server 1");
                server_to_server_socket
                    .send_to(up_index, "127.0.0.4:8080")
                    .await
                    .expect("Failed to send index to server 2");
                server_load_socket
                    .send_to("".as_bytes(), "127.0.0.2:8100")
                    .await
                    .expect("Failed to send index to server 1");
                server_load_socket
                    .send_to("".as_bytes(), "127.0.0.4:8100")
                    .await
                    .expect("Failed to send index to server 2");
                own_down = 0;
                server_down = 0;
                just_up_socket
                    .recv_from(&mut just_up_receive_buffer)
                    .await
                    .expect("Couldn't recieve index");
                current_server = i32::from_be_bytes([
                    just_up_receive_buffer[0],
                    just_up_receive_buffer[1],
                    just_up_receive_buffer[2],
                    just_up_receive_buffer[3],
                ]);
                just_up_receive_buffer = [0; 4];
                skip_socket
                    .recv_from(&mut just_up_receive_buffer)
                    .await
                    .expect("Couldn't recieve index");
                just_slept = i32::from_be_bytes([
                    just_up_receive_buffer[0],
                    just_up_receive_buffer[1],
                    just_up_receive_buffer[2],
                    just_up_receive_buffer[3],
                ]);
                //current_server=1-current_server;
                if (just_slept > 90) {
                    just_slept = 4;
                }
                println!("Current Server Down {}", current_server);
                println!("I am here");
                just_up_receive_buffer = [0; 4];
                //receive_buffer = [0; BUFFER_SIZE];
                //receive_buffer = [0; BUFFER_SIZE];
                num_down += 1;
                if (num_down > 1) {
                    //just_slept=just_slept+1;
                    just_slept = just_slept;
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "127.0.0.3:21111"
        .parse()
        .expect("Failed to parse middleware address");
    let middleware_address_str = middleware_address.to_string();

    // Define the server addresses and middleware addresses
    let server_addresses = ["127.0.0.2:54321", "127.0.0.3:54322", "127.0.0.4:54323"];
    let server3_task = server3("127.0.0.3:54322", &middleware_address_str);

    // Start the server middleware
    let server_middleware_task =
        server_middleware(&middleware_address_str, server_addresses.to_vec());
    let _ = tokio::join!(server3_task, server_middleware_task);
}
