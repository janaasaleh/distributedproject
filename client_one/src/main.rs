use async_std::net::UdpSocket;
use std::fs;
use std::io::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::broadcast;
use tokio::time::sleep;

const BUFFER_SIZE: usize = 140000;
const MAX_PACKET_SIZE: usize = 1400;

async fn send_image(
    client_socket: &UdpSocket,
    image_data: &[u8],
    destination: &SocketAddr,
    max_packet_size: usize,
) -> Result<(), Error> {
    for chunk in image_data.chunks(MAX_PACKET_SIZE) {
        // Send a chunk of the image data
        client_socket
            .send_to(chunk, destination)
            .await
            .expect("Failed to send image chunk");
    }

    Ok(())
}

async fn receive_image(
    client_socket: &UdpSocket,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut image_data: Vec<u8> = Vec::new();
    let mut buffer = [0; BUFFER_SIZE];

    loop {
        match client_socket.recv_from(&mut buffer).await {
            Ok((bytes_received, _)) => {
                // Append the received data to the image_data vector
                image_data.extend_from_slice(&buffer[..bytes_received]);
            }
            Err(_) => {
                // No more data to receive, exit the loop
                break;
            }
        }
    }

    // Write the reconstructed image data to a file
    fs::write(output_path, &image_data)?;

    Ok(())
}

async fn middleware_task(mut middleware_socket: UdpSocket) {
    let server_addresses = ["127.0.0.2:21112", "127.0.0.3:21111", "127.0.0.4:21113"];
    let mut buffer = [0; BUFFER_SIZE];
    let mut ack_buffer = [0; BUFFER_SIZE];
    //let middleware_address: SocketAddr = "127.0.0.8:12345".parse().expect("Failed to parse middleware address");

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
                println!("Yo4");
            }
            println!("Yo5");
            let (ack_bytes_received, server_address) = server_socket
                .recv_from(&mut ack_buffer)
                .await
                .expect("Failed to receive acknowledgment from server");
            middleware_socket
                .send_to(&ack_buffer, client_address)
                .await
                .expect("Failed to send acknowledgment to client");
            //println!("Yo6");

            // Sleep to give time for the server to send the acknowledgment
            sleep(Duration::from_millis(10)).await;

            // Clear the buffer for the next request
            buffer = [0; BUFFER_SIZE];
            ack_buffer = [0; BUFFER_SIZE];
        }
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
    let dos_address = "127.0.0.255:12345";
    let middleware_address: SocketAddr = "127.0.0.8:12345"
        .parse()
        .expect("Failed to parse middleware address");
    let client_socket = UdpSocket::bind("127.0.0.8:0")
        .await
        .expect("Failed to bind client socket");
    //let client_socket_register = UdpSocket::bind("127.0.0.8:8090").await.expect("Failed to bind client socket");
    //let client_socket_query = UdpSocket::bind("127.0.0.8:8091").await.expect("Failed to bind client socket");
    //register_user(client_socket_register,dos_address, "Client1","Client").await;
    //println!("Finished Registry");
    //query_online_users(client_socket_query,dos_address).await;
    let middleware_socket = UdpSocket::bind(&middleware_address)
        .await
        .expect("Failed to bind middleware socket");

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
            let image_data = fs::read("image1.jpg").expect("Failed to read the image file");
            let middleware_address = "127.0.0.8:12345"; // Replace with the actual middleware address and port
                                                        //sleep(Duration::from_millis(5000)).await;
            if let Err(err) = send_image(
                &client_socket,
                &image_data,
                &middleware_address.parse().expect("ay haga"),
                MAX_PACKET_SIZE,
            )
            .await
            {
                eprintln!("Failed to send the image: {}", err);
            } else {
                println!("Image sent successfully to middleware");
            };

            if let Err(err) = receive_image(&client_socket, "encrypted.jpg").await {
                eprintln!("Failed to receive and save the image: {}", err)
            } else {
                println!("Received image saved to 'encrypted.jpg'");
            }
        }
        if input.trim() == "Q" {
            //let unregister_message = "UNREGISTER";
            //let dos_socket = UdpSocket::bind("127.0.0.8:9001").await.expect("Failed to bind socket");
            //dos_socket
            //.send_to(unregister_message.as_bytes(), dos_address_clone)
            //.await
            //.expect("Failed to send unregister message");
            return;
        }
    }

    //middleware_task_handle.await.expect("Middleware task failed");
}
