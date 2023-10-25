use async_std::net::UdpSocket;
use async_std::task;
use std::net::SocketAddr;

async fn server2(server_address: &str, middleware_address: &str) {
    let parts: Vec<&str> = server_address.split(':').collect();
    let port = parts[1].parse::<u16>().expect("Failed to parse port as u16");
    let server_address: SocketAddr = server_address.parse().expect("Failed to parse server address");

    let socket = UdpSocket::bind(&server_address).await.expect("Failed to bind server socket");
    println!("Server 2 socket is listening on {}", server_address);

    let mut buffer = [0; 1024];

    while let Ok((bytes_received, client_address)) = socket.recv_from(&mut buffer).await {
        let message = String::from_utf8_lossy(&buffer);
        println!("H");

        let response = match port {
            54322 => "Server 2 received your message",
            _ => "Server 2 received your message",
        };

        println!("Server 2 received: {}", message);
        println!("Server 2 responding with: {}", response);

        if let Err (err) = socket.send_to(response.as_bytes(), &client_address).await {
            eprintln!("Server 2 failed to send acknowledgment: {}", err);
        }
        println!("{}",client_address);

        //if let Err (err) = socket.send_to(response.as_bytes(), middleware_address).await {
        //    eprintln!("Server 2 failed to send acknowledgment to middleware: {}", err);
        //}
        println!("{}",middleware_address);
    }
}

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "0.0.0.0:12345".parse().expect("Failed to parse middleware address");
    let middleware_address_str = middleware_address.to_string();
    let server2_task = server2("0.0.0.0:54322", &middleware_address_str);

    let _ = tokio::join!(server2_task);
}
