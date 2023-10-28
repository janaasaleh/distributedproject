use async_std::net::UdpSocket;
use async_std::task;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;

async fn server2(server_address: &str, middleware_address: &str, server_addresses: Vec<&str>) {
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
    println!("Server 2 socket is listening on {}", server_address);

    let mut buffer = [0; 1024];

    while let Ok((bytes_received, client_address)) = socket.recv_from(&mut buffer).await {
        let message = String::from_utf8_lossy(&buffer);
        println!("Server 2 received: {}", message);

        let response = match port {
            54322 => "Server 2 received your message",
            _ => "Server 2 received your message",
        };

        println!("Server 2 responding with: {}", response);
        //sleep(Duration::from_millis(7000)).await;
        // Send the response to the client's middleware
        if let Err(err) = socket.send_to(response.as_bytes(), client_address).await {
            eprintln!(
                "Server 2 failed to send acknowledgment to middleware: {}",
                err
            );
        }
        println!("Middleware address {}", client_address);
        // Clear the buffer for the next request
        buffer = [0; 1024];
    }

    // Simulate a server failure (for example, after some time)
    //sleep(Duration::from_secs(10)).await;

    // Notify other servers about the failure
    let server_notify_socket = UdpSocket::bind("127.0.0.3:0")
        .await
        .expect("Failed to bind server notify socket");

    let message = "Server 2 is down";
    for addr in &server_addresses {
        let server_address: SocketAddr = addr.parse().expect("Failed to parse server address");
        if server_address != server_address {
            server_notify_socket
                .send_to(message.as_bytes(), server_address)
                .await
                .expect("Failed to send notification");
        }
    }
}

async fn server_middleware(middleware_address: &str, server_addresses: Vec<&str>) {
    let middleware_socket = UdpSocket::bind(middleware_address)
        .await
        .expect("Failed to bind middleware socket");
    println!("Server middleware is listening on {}", middleware_address);
    let mut current_server = 0;
    let mut receive_buffer = [0; 1024];
    let mut send_buffer = [0; 1024]; // Separate buffer for sending data

    while let Ok((bytes_received, client_address)) =
        middleware_socket.recv_from(&mut receive_buffer).await
    {
        println!("Entered Here 1");

        if (current_server == 0) {
            current_server += 1;
        } else if current_server == 1 {
            current_server += 1;
            continue;
        } else if current_server == 2 {
            current_server = 0;
            continue;
        }

        //continue;
        let server_index = 0; // You can implement load balancing logic here
        let server_address = server_addresses[server_index];
        let server_address: SocketAddr = server_address
            .parse()
            .expect("Failed to parse server address");

        let mut server_socket = UdpSocket::bind("127.0.0.3:0")
            .await
            .expect("Failed to bind server socket");
        server_socket
            .connect(&server_address)
            .await
            .expect("Failed to connect to the server");

        // Copy the received data to the send buffer
        send_buffer[..bytes_received].copy_from_slice(&receive_buffer[..bytes_received]);

        server_socket
            .send_to(&send_buffer[..bytes_received], &server_address)
            .await
            .expect("Failed to send data to server");
        println!("Entered Here 2");

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
        println!("Entered Here 4");

        // Clear the receive buffer for the next request
        receive_buffer = [0; 1024];
    }
}

async fn serverStateNotif(server_address: &str, notify_address: &str) {
    let server_notify_socket = UdpSocket::bind(notify_address)
        .await
        .expect("Failed to bind server notify socket");

    let mut receive_buffer = [0; 1024];

    while let Ok((bytes_received, _)) = server_notify_socket.recv_from(&mut receive_buffer).await {
        let message = String::from_utf8_lossy(&receive_buffer[..bytes_received]);
        println!("Received notification: {}", message);
        //implement failure handling here
        //just logging the message.
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
    let server_notify_address = "127.0.0.5:54322"; // A separate address for server notifications

    // Start the server notification task
    let server_notify_task = serverStateNotif(&server_addresses[1], server_notify_address);

    // Start the server1
    let server2_task = server2(
        "127.0.0.3:54323",
        &middleware_address_str,
        server_addresses.to_vec(),
    );

    // Start the server middleware
    let server_middleware_task =
        server_middleware(&middleware_address_str, server_addresses.to_vec());
    let _ = tokio::join!(server2_task, server_notify_task, server_middleware_task);
}
