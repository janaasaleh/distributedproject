use async_std::net::UdpSocket;
use async_std::task;
use std::net::SocketAddr;

async fn server1(server_address: &str, middleware_address: &str) {
    let parts: Vec<&str> = server_address.split(':').collect();
    let port = parts[1].parse::<u16>().expect("Failed to parse port as u16");
    let server_address: SocketAddr = server_address.parse().expect("Failed to parse server address");

    let socket = UdpSocket::bind(&server_address).await.expect("Failed to bind server socket");
    println!("Server 1 socket is listening on {}", server_address);

    let mut buffer = [0; 1024];

    while let Ok((bytes_received, client_address)) = socket.recv_from(&mut buffer).await {
        let message = String::from_utf8_lossy(&buffer);
        println!("H");

        let response = match port {
            54321 => "Server 1 received your message",
            _ => "Server 1 received your message",
        };

        println!("Server 1 received: {}", message);
        println!("Server 1 responding with: {}", response);

        if let Err(err) = socket.send_to(response.as_bytes(), &client_address).await {
            eprintln!("Server 1 failed to send acknowledgment: {}", err);
        }
        println!("{}",client_address);

        //if let Err(err) = socket.send_to(response.as_bytes(), middleware_address).await {
       //     eprintln!("Server 1 failed to send acknowledgment to middleware: {}", err);
        //}
        println!("Server 1 acknowledging with: {}", middleware_address);
    }
}

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "0.0.0.0:12345".parse().expect("Failed to parse middleware address");
    let middleware_address_str = middleware_address.to_string();
    let server1_task = server1("0.0.0.0:54321", &middleware_address_str);

    let _ = tokio::join!(server1_task);
}
