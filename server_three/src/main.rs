use async_std::net::UdpSocket;
use std::net::SocketAddr;

async fn server3(server_address: &str, _middleware_address: &str) {
    let parts: Vec<&str> = server_address.split(':').collect();
    let port = parts[1].parse::<u16>().expect("Failed to parse port as u16");
    let server_address: SocketAddr = server_address.parse().expect("Failed to parse server address");

    let socket = UdpSocket::bind(&server_address).await.expect("Failed to bind server socket");
    println!("Server 3 socket is listening on {}", server_address);

    let mut buffer = [0; 1024];

    while let Ok((_bytes_received, client_address)) = socket.recv_from(&mut buffer).await {
        let message = String::from_utf8_lossy(&buffer);
        println!("Server 3 received: {}", message);

        let response = match port {
            54323 => "Server 3 received your message",
            _ => "Server 3 received your message",
        };

        println!("Server 3 responding with: {}", response);
        //sleep(Duration::from_millis(10000)).await;
       
        // Send the response to the client's middleware
        if let Err(err) = socket.send_to(response.as_bytes(), client_address).await {
            eprintln!("Server 3 failed to send acknowledgment to middleware: {}", err);
        }
        //println!("Middleware address {}",client_address);
        // Clear the buffer for the next request
        buffer = [0; 1024];
    }
}

async fn server_middleware(middleware_address: &str, server_addresses: Vec<&str>) {
    let middleware_socket = UdpSocket::bind(middleware_address).await.expect("Failed to bind middleware socket");
    let server_to_server_socket = UdpSocket::bind("127.0.0.4:8080")
    .await
    .expect("Failed to bind server to server socket");

    println!("Server middleware is listening on {}", middleware_address);

    let mut receive_buffer = [0; 1024];
    let mut current_server = 0;
    let mut send_buffer = [0; 1024]; // Separate buffer for sending data
    while let Ok((bytes_received, client_address)) = middleware_socket.recv_from(&mut receive_buffer).await {
        println!("Entered Here 1");

        let mut server_to_server_receive_buffer = [0; 4];

        server_to_server_socket
            .recv_from(&mut server_to_server_receive_buffer)
            .await
            .expect("Couldn't recieve index");

        let index = i32::from_be_bytes([
            server_to_server_receive_buffer[0],
            server_to_server_receive_buffer[1],
            server_to_server_receive_buffer[2],
            server_to_server_receive_buffer[3],
        ]);

        println!("Index recieved {}", index);

        if index != current_server {
            current_server = index;
        }

        if current_server==0
        {
            current_server+=1;
            continue;
        }
        else if current_server == 1
        {
            current_server+=1;
            continue;
        }
        else if current_server == 2
        {
            current_server=0;
        }
        let server_index = 2;  // You can implement load balancing logic here
        let server_address = server_addresses[server_index];
        let server_address: SocketAddr = server_address.parse().expect("Failed to parse server address");

        let server_socket = UdpSocket::bind("127.0.0.4:0").await.expect("Failed to bind server socket");
        server_socket.connect(&server_address).await.expect("Failed to connect to the server");
       
        // Copy the received data to the send buffer
        send_buffer[..bytes_received].copy_from_slice(&receive_buffer[..bytes_received]);

        server_socket.send_to(&send_buffer[..bytes_received], &server_address).await.expect("Failed to send data to server");
        println!("Entered Here 2");

        let (ack_bytes_received, server_caddress) = server_socket.recv_from(&mut receive_buffer).await.expect("Failed to receive acknowledgment from server");
        println!("Entered Here 3");
        //println!("Server address {}",server_caddress);

        // Send the acknowledgment from the server to the client's middleware
        middleware_socket.send_to(&receive_buffer[..ack_bytes_received], client_address).await.expect("Failed to send acknowledgment to client");
        println!("Entered Here 4");

        // Clear the receive buffer for the next request
        server_to_server_receive_buffer = [0;4];
        receive_buffer = [0; 1024];
    }
}


#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "127.0.0.4:21113".parse().expect("Failed to parse middleware address");
    let middleware_address_str = middleware_address.to_string();
   
    // Define the server addresses and middleware addresses
    let server_addresses = ["127.0.0.2:54321", "127.0.0.3:54322","127.0.0.4:54323"];
    let server3_task = server3("127.0.0.4:54323", &middleware_address_str);
   
    // Start the server middleware
    let server_middleware_task = server_middleware(&middleware_address_str, server_addresses.to_vec());
    let _ = tokio::join!(server3_task, server_middleware_task);
}



