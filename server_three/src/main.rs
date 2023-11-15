use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use async_std::net::UdpSocket;
use tokio::time::{Duration, Instant};
use image::GenericImageView; 

use std::net::SocketAddr;

const BUFFER_SIZE: usize = 140000;
const MAX_PACKET_SIZE: usize = 1400;
const ELECTION_TIMEOUT: Duration = Duration::from_secs(10);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone)]
struct ServerState {
    is_leader: bool,
    is_active: bool,
    active_servers: HashSet<String>,
}

async fn send_image(
    socket: &UdpSocket,
    image_data: &[u8],
    destination: &SocketAddr,
    max_packet_size: usize,
) -> Result<(), std::io::Error> {
    for chunk in image_data.chunks(max_packet_size) {
        socket
            .send_to(chunk, destination)
            .await
            .expect("Failed to send image chunk");
    }

    Ok(())
}

async fn server3(server_address: &str, middleware_address: &SocketAddr, state: Arc<Mutex<ServerState>>) {
    let parts: Vec<&str> = server_address.split(':').collect();
    let port = parts[1]
        .parse::<u16>()
        .expect("Failed to parse port as u16");
    let server_address: SocketAddr = server_address
        .parse()
        .expect("Failed to parse server address");

    let socket = UdpSocket::bind(&server_address)
        .await
        .expect("Failed to bind server socket");

    let mut buffer = [0; BUFFER_SIZE];
    let mut image_chunks: Vec<u8> = Vec::new();

    while let Ok((_bytes_received, client_address)) = socket.recv_from(&mut buffer).await {
        let chunk = &buffer[.._bytes_received];
        image_chunks.extend_from_slice(chunk);

        if chunk.len() < BUFFER_SIZE {
            let image_result = image::load_from_memory(&image_chunks);
            if let Ok(mut image) = image_result {
                if let Err(err) =
                    send_image(&socket, &image_chunks, middleware_address, MAX_PACKET_SIZE).await
                {
                    eprintln!(
                        "Server 3 failed to send processed image to middleware: {}",
                        err
                    );
                } else {
                    println!("Server 3 sent processed image back to middleware");
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
                "Server 3 failed to send acknowledgment to middleware: {}",
                err
            );
        }
        println!("Middleware address {}", client_address);
        buffer = [0; BUFFER_SIZE];
    }
}

async fn server_middleware(
    middleware_address: &SocketAddr,
    server_addresses: Vec<&str>,
    state: Arc<Mutex<ServerState>>,
) {
    let middleware_socket = UdpSocket::bind(middleware_address)
        .await
        .expect("Failed to bind middleware socket");

        //et middleware_socket = Arc::new(Mutex::new(UdpSocket::bind(middleware_address).await.unwrap()));
    

    let server_to_server_socket = UdpSocket::bind("127.0.0.4:8080")
        .await
        .expect("Failed to bind server to server socket");

    server_to_server_socket
        .connect("127.0.0.3:8080")
        .await
        .expect("Failed to connect to Server 2");
    server_to_server_socket
        .connect("127.0.0.2:8080")
        .await
        .expect("Failed to connect to Server 1");

    println!("Server middleware is listening on {}", middleware_address);

    let mut current_server: i32 = 2;
    let mut receive_buffer = [0; BUFFER_SIZE];
   
   
    
    while let Ok((bytes_received, client_address)) = middleware_socket.recv_from(&mut receive_buffer).await {
        println!("Entered Here 1");
        println!("Received message from client");
        if state.lock().unwrap().is_active {
        
        let message = String::from_utf8_lossy(&receive_buffer[..bytes_received]);
        println!("Client message: {}", message);

        // Handle client request (example: just echoing back the message)
        let response_message = format!("Server received: {}", message);

        // Send the response back to the client
        middleware_socket
            .send_to(response_message.as_bytes(), client_address)
            .await
            .expect("Failed to send response to client");
       


        println!("Entered Here 1");
    } else {
        // If the server is not active, send the client request to the next active server
        if !state.lock().unwrap().is_leader {
            startelection(
                server_addresses.clone(),
                state.clone(),
                &server_to_server_socket,
            )
            .await;
        } else {
            redistribute_workload(state.clone(), server_addresses.clone(), &server_to_server_socket,&receive_buffer, bytes_received).await;
        }

    // if state.lock().unwrap().is_leader {
    //     redistribute_workload(state.clone(), server_addresses.clone(),&server_to_server_socket).await;
    // }

    println!("Entered Here 1");
}
}
}








async fn redistribute_workload(
    state: Arc<Mutex<ServerState>>,
    server_addresses: Vec<&str>,
    server_to_server_socket: &UdpSocket,
    receive_buffer: &[u8],
    bytes_received: usize,
) {
    let my_address = state.lock().unwrap().active_servers.iter().next().unwrap().to_string();
    let active_servers = state.lock().unwrap().active_servers.clone();
    let mut server_iter = server_addresses.iter().cycle().skip_while(|&&addr| addr != my_address);

    // Find the next active server in a cyclic manner
    let next_active_server = server_iter.find(|&&addr| active_servers.contains(addr));

    if let Some(server_address) = next_active_server {
        let workload_message = format!("request redistributed to {}", server_address);

    if let Some(&server_address) = next_active_server {
        let workload_message = format!("Workload redistributed to {}", server_address);

        // Use the appropriate server-to-server socket based on the server address
        match server_address {
            "127.0.0.2:54321" => {
                server_to_server_socket
                    .send_to(workload_message.as_bytes(), "127.0.0.2:8080")
                    .await
                    .expect("Failed to redistribute workload");
            }
            "127.0.0.3:54322" => {
                server_to_server_socket
                    .send_to(workload_message.as_bytes(), "127.0.0.3:8080")
                    .await
                    .expect("Failed to redistribute workload");
            }
            _ => {
                // Handle the case where the next active server is not Server 2 or Server 3
                println!("Unknown server address: {}", server_address);
            }
          
        }
    } else {
        // Handle the case where there is no next active server
        println!("No next active server found.");
    }
}
}

async fn startelection(
    server_addresses: Vec<&str>,
    state: Arc<Mutex<ServerState>>,
    server_to_server_socket:&UdpSocket,
) {
    let my_address = server_addresses[2];
    let my_index: i32 = my_address.chars().last().unwrap().to_digit(10).unwrap() as i32;

    for &server_address in server_addresses.iter() {
        let server_index: i32 = server_address.chars().last().unwrap().to_digit(10).unwrap() as i32;
        if server_index == my_index {
            continue;
        }
        if server_index > my_index {
            let election_message = format!("ELECTION {}", my_index);

            // Use the appropriate server-to-server socket based on the server index
            match server_index {
                2 => {
                    server_to_server_socket
                        .send_to(election_message.as_bytes(),"127.0.0.3:8080")
                        .await
                        .expect("Failed to send election message");
                }
                _ => {
                    // Use the default socket for Server 1 (index 1)
                    server_to_server_socket
                        .send_to(election_message.as_bytes(), "127.0.0.2:8080")
                        .await
                        .expect("Failed to send election message");
                }
            }
        }

        
    }

    let mut receive_buffer = [0; BUFFER_SIZE];
    let mut ack_received = false;
    let timeout = Instant::now() + ELECTION_TIMEOUT;

    while Instant::now() < timeout {
        match server_to_server_socket.recv_from(&mut receive_buffer).await {
            Ok((bytes_received, _)) => {
                let message = &receive_buffer[..bytes_received];
                if message == b"ELECTION_ACK" {
                    ack_received = true;
                    break;
                }
            }
            Err(_) => {
                eprintln!("Error receiving acknowledgment message from Servers");
            }
        }
    
    if ack_received {
        state.lock().unwrap().is_leader = false;
    } else {
        state.lock().unwrap().is_leader = true;
    }

    println!(
        "Server {} completed the electionand is the leader: {}",
        my_index,
        state.lock().unwrap().is_leader
    );
}
}

use std::thread;

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "127.0.0.4:21113"
        .parse()
        .expect("Failed to parse middleware address");

    let server_addresses = ["127.0.0.2:54321", "127.0.0.3:54322", "127.0.0.4:54323"];
    let state = Arc::new(Mutex::new(ServerState {is_leader:false,active_servers:HashSet::new(), is_active: true }));

  

let builder = thread::Builder::new().stack_size(32 * 1024 * 1024); // Set a larger stack size (e.g., 32MB)
    builder.spawn(move || {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let server3_task = server3("127.0.0.4:54323", &middleware_address, state.clone());
            let server_middleware_task =
                server_middleware(&middleware_address, server_addresses.to_vec(), state.clone());
            let _ = tokio::join!(server3_task, server_middleware_task);
        });
    }).unwrap().join().unwrap();
}
