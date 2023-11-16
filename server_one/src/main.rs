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

            let image_string = base64::encode(image_data.clone());
            let payload = str_to_bytes(&image_string);
            let destination_image = file_as_dynamic_image("encrypt.png".to_string());
            let enc = Encoder::new(payload, destination_image);
            let result = enc.encode_alpha();
            save_image_buffer(result, "encrypted.png".to_string());

            // let encoded_image = file_as_image_buffer("encrypted.png".to_string());
            // let dec = Decoder::new(encoded_image);
            // let out_buffer = dec.decode_alpha();
            // let clean_buffer: Vec<u8> = out_buffer.into_iter().filter(|b| *b != 0xff_u8).collect();
            // let message = bytes_to_str(clean_buffer.as_slice());

            // let decoded_image_data = base64::decode(message).unwrap_or_else(|e| {
            //     eprintln!("Error decoding base64: {}", e);
            //     Vec::new()
            // });

            // if let Ok(decoded_image) = image::load_from_memory(&decoded_image_data) {
            //     let (width, height) = decoded_image.dimensions();
            //     println!("Image dimensions: {} x {}", width, height);

            //     if let Err(err) = decoded_image.save("decoded.png") {
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
    let server_to_server_socket = UdpSocket::bind("127.0.0.2:8080")
        .await
        .expect("Failed to bind server to server socket");
    let just_up_socket = UdpSocket::bind("127.0.0.2:8087")
        .await
        .expect("Failed to bind server to server socket");
    let skip_socket = UdpSocket::bind("127.0.0.2:8088")
        .await
        .expect("Failed to bind server to server socket");
    let server_load_socket = UdpSocket::bind("127.0.0.2:8100")
        .await
        .expect("Failed to bind server to server socket");
    let packets_size_socket = UdpSocket::bind("127.0.0.2:8101")
        .await
        .expect("Failed to bind server to server socket");

    let mut server_down=0;
    let real_server_down=0;
    let server_down_index:i32=10;
    let server_up_index:i32=20;
    let mut own_down:i32=0;
    let mut which_server:i32=500;
    let mut just_up:i32=0;
    let mut previous_down:i32=0;
    let mut just_slept:i32=0;
    let mut server_down_requests:i32=0;
    let mut num_down:i32=0;
    println!("Server middleware is listening on {}", middleware_address);
    let mut current_server:i32 = 0;
    let mut receive_buffer = [0; 1024];
    let mut send_buffer = [0; 1024]; // Separate buffer for sending data
    let mut my_load=String::from("");  //New
    let mut _my_packets=0;  //New
    let mut current_packet=0;  //New
   
    while let Ok((bytes_received, client_address)) =
        middleware_socket.recv_from(&mut receive_buffer).await
    {
        let ip= client_address.ip();       //New
        let ip_string = ip.to_string();        // New
         
        println!("Just Slept:{}",just_slept);
        if(just_slept==1)
        {
            just_slept=0;
            continue;
        }
        else if just_slept>1
        {
            just_slept=just_slept-1;
            continue;
        }
        else {
            just_slept=0;
        }
        if(which_server==1)
        {
            server_down_requests+=1;
        }
        if(which_server==500)
        {
            server_down_requests=0;
        }
        println!("Current Server: {}",current_server);
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
        let mut server1_load_receive_buffer = [0; 1024];
        let mut server2_load_receive_buffer = [0; 1024];
        let mut packets_size_receive_buffer = [0; 4];


        //server_to_server_socket
        //.connect("127.0.0.3:8080")
        //.await
        //.expect("Failed to connect to the server");
//
        //server_to_server_socket
        //.connect("127.0.0.4:8080")
        //.await
        //.expect("Failed to connect to the server");
        if random_number < 0 && server_down==0 && real_server_down==0 && previous_down==0 && my_load==""{
        server_down=0;
        own_down=1;
        server_to_server_socket
            .send_to(down_index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 2");
        server_to_server_socket
            .send_to(down_index, "127.0.0.4:8080")
            .await
            .expect("Failed to send index to server 3");
        server_load_socket
            .send_to(my_load.as_bytes(), "127.0.0.4:8100")
            .await
            .expect("Failed to send index to server 3");
        server_load_socket
            .send_to(my_load.as_bytes(), "127.0.0.4:8100")
            .await
            .expect("Failed to send index to server 3");
        println!("Server 1 going down!");
        }
        else {
            if(which_server==500)
            {
            server_to_server_socket
            .send_to(index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 2");
            server_to_server_socket
            .send_to(index, "127.0.0.4:8080")
            .await
            .expect("Failed to send index to server 3");
            server_load_socket
            .send_to(my_load.as_bytes(), "127.0.0.4:8100")
            .await
            .expect("Failed to send index to server 3");
            server_load_socket
            .send_to(my_load.as_bytes(), "127.0.0.4:8100")
            .await
            .expect("Failed to send index to server 3");
            }
            else if which_server==1
            {
                server_to_server_socket
            .send_to(index, "127.0.0.4:8080")
            .await
            .expect("Failed to send index to server 3");
            server_load_socket
            .send_to(my_load.as_bytes(), "127.0.0.4:8100")
            .await
            .expect("Failed to send index to server 3");
            }
            else if which_server==2
            {
            server_to_server_socket
            .send_to(index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 2");
            server_load_socket
            .send_to(my_load.as_bytes(), "127.0.0.4:8100")
            .await
            .expect("Failed to send index to server 3");
            }
        }


        server_load_socket
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

       
        let otherload1 = &String::from_utf8_lossy(&server1_load_receive_buffer).to_string();

        if(server_down==0 || index1>19)
        {
        server_to_server_socket
            .recv_from(&mut server_to_server2_receive_buffer)
            .await
            .expect("Couldn't recieve index");
        server_load_socket
            .recv_from(&mut server2_load_receive_buffer)
            .await
            .expect("Couldn't recieve index");
        }

        let mut index2 = i32::from_be_bytes([
            server_to_server2_receive_buffer[0],
            server_to_server2_receive_buffer[1],
            server_to_server2_receive_buffer[2],
            server_to_server2_receive_buffer[3],
        ]);
        let otherload2 = &String::from_utf8_lossy(&server2_load_receive_buffer);

        println!("Index recieved {}", index1);
        println!("Index recieved {}", index2);

        if(index1>9 && index1<20 && index2>9 && index2<20 && own_down==1)
        {
            index1=current_server;
            index2=current_server;
            own_down=0;
            println!("All 3 servers requested sleep. Overturned!")
        }
        else if((own_down==1 && index1>9 && index1<20))
        {
            own_down=0;
            index1=current_server;
            println!("2 servers requested sleep. Overturned!")
        }
        else if((own_down==1 && index2>9 && index2<20))
        {
            own_down=0;
            index2=current_server;
            println!("2 servers requested sleep. Overturned!")
        }
        else if(index1>9 && index1<20 && index2>9 && index2<20 && own_down==0)
        {
            index1=current_server;
            index2=current_server;
            println!("2 servers not me requested sleep. Overturned!")
        }

        if (index1>9)
        {
            if index1<19
            {
                if(index1 == 11)
                {
                    println!("Server 2 is down");
                    server_down=1;
                    which_server=1;
                }
                if(index1 == 12)
                {
                    println!("Server 3 is down");
                    server_down=1;
                    which_server=2;
                }

            }
            else {
                if index1==21
                {
                    println!("Server 2 is back up");
                    server_down=0;
                    which_server=500;
                    just_up=1;
                }
                if index1==22
                {
                    println!("Server 3 is back up");
                    server_down=0;
                    which_server=500;
                }
               
            }

        }
        if (index2>9)
        {
            if index2<19
            {
                if(index2 == 11)
                {
                    println!("Server 2 is down");
                    server_down=1;
                    which_server=1;
                }
                if(index2 == 12)
                {
                    println!("Server 3 is down");
                    server_down=1;
                    which_server=2;
                }

            }
            else {
                if index2==21
                {
                    println!("Server 2 is back up");
                    server_down=0;
                    which_server=500;
                    just_up=1;
                }
                if index2==22
                {
                    println!("Server 3 is back up");
                    server_down=0;
                    which_server=500;
                }
               
            }

        }

        if(which_server==500)
        {
        if current_server == 0 {
            current_server += 1;
            if(just_up==1)
            {
            let cs: &[u8] = &current_server.to_be_bytes();
            server_to_server_socket
            .send_to(cs, "127.0.0.3:8087")
            .await.expect("Failed to send index to server 2");
            let sdr: &[u8] = &server_down_requests.to_be_bytes();
            server_to_server_socket
            .send_to(sdr, "127.0.0.3:8088")
            .await.expect("Failed to send index to server 3");
            just_up=0;
            }
            if(ip_string==my_load)
            {
            }
            else if my_load=="" && &ip_string!=otherload1 && &ip_string!=otherload2
            {
                let ip_strings: String = ip.to_string();
                my_load=ip_strings;
                packets_size_socket
                .recv_from(&mut packets_size_receive_buffer)
                .await
                .expect("Couldn't recieve index");
                // _my_packets = i32::from_be_bytes([
                //packets_size_receive_buffer[0],
                //packets_size_receive_buffer[1],
                //packets_size_receive_buffer[2],
                //packets_size_receive_buffer[3],
            //]);
            }
            else {
                continue;
            }
           
        } else if current_server == 1 {
            current_server += 1;
            if(just_up==1)
            {
            let cs: &[u8] = &current_server.to_be_bytes();
            server_to_server_socket
            .send_to(cs, "127.0.0.3:8087")
            .await.expect("Failed to send index to server 2");
            let sdr: &[u8] = &server_down_requests.to_be_bytes();
            server_to_server_socket
            .send_to(sdr, "127.0.0.3:8088")
            .await.expect("Failed to send index to server 3");
            just_up=0;
            }
            if(own_down==1)
            {
            tokio::time::sleep(Duration::from_secs(40)).await;
            previous_down=6;
            println!("Server 1 going up!");
            server_to_server_socket
            .send_to(up_index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 2");
            server_to_server_socket
            .send_to(up_index, "127.0.0.4:8080")
            .await
            .expect("Failed to send index to server 3");
            own_down=0;
            server_down=0;
            just_up_socket
            .recv_from(&mut just_up_receive_buffer)
            .await
            .expect("Couldn't recieve index");
            current_server=i32::from_be_bytes([
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
            just_slept=i32::from_be_bytes([
            just_up_receive_buffer[0],
            just_up_receive_buffer[1],
            just_up_receive_buffer[2],
            just_up_receive_buffer[3],
            ]);
            //current_server=1-current_server;
            if(just_slept>90)
            {
                just_slept=93;
            }
            println!("Current Server Down {}",current_server);
            println!("I am here");
            just_up_receive_buffer = [0; 4];
            //receive_buffer = [0; 1024];
            //receive_buffer = [0; 1024];
            num_down+=1;
            if(num_down>1)
            {
                //just_slept=just_slept+1;
                just_slept=just_slept;
            }
            }
            if(ip_string==my_load)
            {
            }
            else {
            continue;
            }
        } else if current_server == 2{
            current_server = 0;
            if(just_up==1)
            {
            let cs: &[u8] = &current_server.to_be_bytes();
            server_to_server_socket
            .send_to(cs, "127.0.0.3:8087")
            .await.expect("Failed to send index to server 2");
            let sdr: &[u8] = &server_down_requests.to_be_bytes();
            server_to_server_socket
            .send_to(sdr, "127.0.0.3:8088")
            .await.expect("Failed to send index to server 3");
            just_up=0;
            }
            if(own_down==1)
            {
                tokio::time::sleep(Duration::from_secs(40)).await;
                previous_down=6;
                println!("Server 1 going up!");
                server_to_server_socket
                .send_to(up_index, "127.0.0.3:8080")
                .await
                .expect("Failed to send index to server 2");
                server_to_server_socket
                .send_to(up_index, "127.0.0.4:8080")
                .await
                .expect("Failed to send index to server 3");
                own_down=0;
                server_down=0;
                just_up_socket
                .recv_from(&mut just_up_receive_buffer)
                .await
                .expect("Couldn't recieve index");
                current_server=i32::from_be_bytes([
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
                just_slept=i32::from_be_bytes([
                just_up_receive_buffer[0],
                just_up_receive_buffer[1],
                just_up_receive_buffer[2],
                just_up_receive_buffer[3],
                ]);
                if(just_slept>90)
            {
                just_slept=93;
            }
                //current_server=1-current_server;
                println!("Current Server Down {}",current_server);
                println!("I am here");
                just_up_receive_buffer = [0; 4];
                //receive_buffer = [0; 1024];
                //receive_buffer = [0; 1024];
                num_down+=1;
                if(num_down>1)
                {
                    //just_slept=just_slept+1;
                    just_slept=just_slept;
                }
            }
            if(ip_string==my_load)
            {
            }
            else {
            continue;
            }
        }
    }
    else if (which_server==1)
    {

        if current_server == 0{
            current_server += 2;
            if(ip_string==my_load)
            {
            }
            else if my_load=="" && &ip_string!=otherload1 && &ip_string!=otherload2
            {
                let ip_strings: String = ip.to_string();
                my_load=ip_strings;
                //packets_size_socket
                //.recv_from(&mut packets_size_receive_buffer)
                //.await
                //.expect("Couldn't recieve index");
                //_my_packets = i32::from_be_bytes([
                //packets_size_receive_buffer[0],
                //packets_size_receive_buffer[1],
                //packets_size_receive_buffer[2],
                //packets_size_receive_buffer[3],
            //]);
            }
            else {
                continue;
            }

        } else if current_server == 1 {
            current_server += 1;
            if(ip_string==my_load)
            {
            }
            else {
            continue;
            }
        } else if current_server == 2 {
            current_server = 0;
            if(ip_string==my_load)
            {
            }
            else {
            continue;
            }
        }
    }
    else if (which_server==2)
    {
        if current_server == 0 {
            current_server += 1;
            if(ip_string==my_load)
            {
            }
            else if my_load=="" && &ip_string!=otherload1 && &ip_string!=otherload2
            {
                let ip_strings: String = ip.to_string();
                my_load=ip_strings;
               //packets_size_socket
               //.recv_from(&mut packets_size_receive_buffer)
               //.await
               //.expect("Couldn't recieve index");
               //_my_packets = i32::from_be_bytes([
               //packets_size_receive_buffer[0],
               //packets_size_receive_buffer[1],
               //packets_size_receive_buffer[2],
               //packets_size_receive_buffer[3],
            //]);
            }
            else {
                continue;
            }
        } else if current_server == 1{
            current_server = 0;
            if(ip_string==my_load)
            {
            }
            else {
            continue;
            }
        } else if current_server == 2 {
            current_server = 0;
            if(ip_string==my_load)
            {
            }
            else {
            continue;
            }
        }
    }
    else {
        println!("No which server variable");
    }

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
        shift_left(send_buffer, bytes_received);
        println!("Entered Here 2");
        if(current_packet==_my_packets)
        {
        for i in 1.._my_packets
        {
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
        shift_left(receive_buffer, ack_bytes_received);
        println!("Entered Here 4");
        println!("Client Address:{}",client_address);
        }
        }
        else
        { 
        
        current_packet+=1;

        // Clear the receive buffer for the next request
        server_to_server1_receive_buffer = [0; 4];
        server_to_server2_receive_buffer = [0; 4];
        receive_buffer = [0; 1024];
        if(own_down==1)
        {
            tokio::time::sleep(Duration::from_secs(40)).await;
            previous_down=6;
            println!("Server 1 going up!");
            server_to_server_socket
            .send_to(up_index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 1");
            server_to_server_socket
            .send_to(up_index, "127.0.0.4:8080")
            .await
            .expect("Failed to send index to server 2");
            own_down=0;
            server_down=0;
            server_to_server_socket
            .recv_from(&mut just_up_receive_buffer)
            .await
            .expect("Couldn't recieve index");
            current_server=i32::from_be_bytes([
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
            just_slept=i32::from_be_bytes([
            just_up_receive_buffer[0],
            just_up_receive_buffer[1],
            just_up_receive_buffer[2],
            just_up_receive_buffer[3],
            ]);
            //current_server=1-current_server;
            if(just_slept>90)
            {
                just_slept=93;
            }
            println!("Current Server Down {}",current_server);
            println!("I am here");
            just_up_receive_buffer = [0; 4];
            //receive_buffer = [0; 1024];
            //receive_buffer = [0; 1024];
            num_down+=1;
            if(num_down>1)
            {
                //just_slept=just_slept+1;
                just_slept=just_slept;
            }
        }
        just_up_receive_buffer = [0; 4];
    }
    }
}

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
