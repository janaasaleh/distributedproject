use async_std::net::UdpSocket;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::signal::unix::{signal, SignalKind};
use tokio::time::sleep;
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

async fn middleware_task(mut middleware_socket: UdpSocket) {
    let server_addresses = ["127.0.0.2:21112", "127.0.0.3:21111", "127.0.0.4:21113"];
    let mut buffer = [0; 1024];
    let mut ack_buffer = [0; 1024];
    //let middleware_address: SocketAddr = "127.0.0.8:12345".parse().expect("Failed to parse middleware address");
   
    // let vector = 0
    let mut code_zero:String=String::new();
    code_zero="AA".to_string();

    loop {
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
                //server_socket
                //    .connect(&server_address)
                //    .await
                //    .expect("Failed to connect to the server");
                println!("Yo3");
                server_socket
                    .send_to(&buffer, &server_address)
                    .await
                    .expect("Failed to send data to server");
                shift_left(&mut buffer, bytes_received);
            }
            
            let timeout_duration = Duration::from_secs(1);

            let message = String::from_utf8_lossy(&buffer);

            match timeout(timeout_duration, server_socket.recv_from(&mut ack_buffer)).await {
                Ok(Ok((ack_bytes_received, server_address))) => {
                middleware_socket
                .send_to(&ack_buffer, client_address)
                .await
                .expect("Failed to send acknowledgment to client");
                shift_left(&mut ack_buffer, ack_bytes_received);
                }
                Ok(Err(e)) => {
                }
                Err(_) => {
                code_zero = "".to_string();
                middleware_socket
                .send_to(code_zero.as_bytes(), client_address)
                .await
                .expect("Failed to send acknowledgment to client");
                code_zero = "AA".to_string();
                }
            }

            //println!("Yo6");

            // Sleep to give time for the server to send the acknowledgment
            sleep(Duration::from_millis(10)).await;

            // Clear the buffer for the next request
            buffer = [0; 1024];
            ack_buffer = [0; 1024];
        }
        }
    }

async fn register_user(client_socket: UdpSocket, dos_address: &str, username: &str,usertype: &str) {
    let registration_message = format!("REGISTER:{}:{}", username,usertype);
    client_socket.send_to(registration_message.as_bytes(), dos_address).await.expect("Failed to send registration request");

    let mut response_buffer = [0; 1024];
    let (bytes_received, _dos_address) = client_socket.recv_from(&mut response_buffer).await.expect("Failed to receive response");
    let response = String::from_utf8_lossy(&response_buffer[..bytes_received]);
    println!("Registration response: {}", response);
}

async fn query_online_users(client_socket: UdpSocket, middleware_address: &str) {
    // Send a query message to request the list of online users
    client_socket.send_to("QUERY".as_bytes(), middleware_address).await.expect("Failed to send query request");

    let mut response_buffer = [0; 1024];
    let (bytes_received, _middleware_address) = client_socket.recv_from(&mut response_buffer).await.expect("Failed to receive response");
    let response = String::from_utf8_lossy(&response_buffer[..bytes_received]);
    println!("Online users: {}", response);
}
   

#[tokio::main]
async fn main() {
    let dos_address= "127.0.0.255:12345";
    let middleware_address: SocketAddr = "127.0.0.8:12345".parse().expect("Failed to parse middleware address");
    let client_socket = UdpSocket::bind("127.0.0.8:0").await.expect("Failed to bind client socket");
    //let client_socket_register = UdpSocket::bind("127.0.0.8:8090").await.expect("Failed to bind client socket");
    //let client_socket_query = UdpSocket::bind("127.0.0.8:8091").await.expect("Failed to bind client socket");
    //register_user(client_socket_register,dos_address, "Client1","Client").await;
    //println!("Finished Registry");
    //query_online_users(client_socket_query,dos_address).await;
    let middleware_socket = UdpSocket::bind(&middleware_address).await.expect("Failed to bind middleware socket");

    tokio::spawn(middleware_task(middleware_socket));
    let dos_address_clone = dos_address.clone(); // Assuming `dos_address` is defined elsewhere
    let termination = Arc::new(Mutex::new(0));
    let termination_clone = Arc::clone(&termination);

    let (tx, _) = tokio::sync::broadcast::channel::<()>(1);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    let mut signal = signal(SignalKind::interrupt()).expect("Failed to create signal handler");
    tokio::spawn(async move {
        signal.recv().await;
        println!("Received termination signal");

        //let unregister_message = "UNREGISTER";
        //let dos_socket = UdpSocket::bind("127.0.0.8:9000").await.expect("Failed to bind socket");
        //dos_socket
        //    .send_to(unregister_message.as_bytes(), dos_address_clone)
        //    .await
        //    .expect("Failed to send unregister message");
//
        // Notify other tasks waiting for the signal
        //let _ = tx.send(());
        *termination_clone.lock().unwrap() = 1;
        tokio::time::sleep(Duration::from_secs(2)).await;

        // Exit the application
        std::process::exit(0);
        //return;
    });

    while *termination.lock().unwrap() == 0 {
        // Code to trigger a request, perhaps on a button press or any other event
        // For demonstration purposes, a message is sent when 'Enter' key is pressed
        println!("Press Enter to send a Request");
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        if input.trim() == "" {
            let mut client_buffer = [0; 1024];
            

            println!("Sending Image");
            let image_data = fs::read("image.png").expect("Failed to read the image file");
            let middleware_address = "0.0.0.0:12345"; // Replace with the actual middleware address and port
                                                      //sleep(Duration::from_millis(5000)).await;
            
            let packet_number = image_data.len() / MAX_CHUNCK;
            let mut i = 1;
            for (index, piece) in image_data.chunks(MAX_CHUNCK).enumerate() {
                let is_last_piece = index == packet_number;
                let chunk = Chunk {
                    position: if is_last_piece { -1 } else { i },
                    packet: {
                        let mut packet_array = [0; MAX_CHUNCK];
                        packet_array[..piece.len()].copy_from_slice(piece);
                        packet_array
                    },
                };

                i += 1;

                let serialized = serde_json::to_string(&chunk).unwrap();

                if(i==packet_number)
                {
                for j in 0..packet_number
                {
                   client_socket
                   .recv_from(&mut client_buffer)
                   .await
                   .expect("Failed to receive response from server");
                   let response= String::from_utf8_lossy(&client_buffer);
                   image_data.push(response.to_string());
                   // Form Image
                   if(j==packet_number) {
                    println!("Image formed");
                    image_data.clear();
                    i=i+1;
                   }
                   
                }
                }
                else 
                {
                client_socket
                .recv_from(&mut client_buffer)
                .await
                .expect("Failed to receive response from server");
                let response = String::from_utf8_lossy(&client_buffer);
                if(response=="".to_string())
                {
                    i=i-1;
                }
                else {
                    i=i+1;
                }
                println!("Client received response from server: {}", response);
                }
    
                    client_socket
                        .send_to(&serialized.as_bytes(), middleware_address)
                        .await
                        .expect("Failed to send piece to middleware");
    
                    let mut ack_buffer = [0; BUFFER_SIZE];
                    client_socket
                        .recv_from(&mut ack_buffer)
                        .await
                        .expect("Failed to receive acknowledgment");
            }
            println!("Sent {} packets", i);
        }
    }
}
