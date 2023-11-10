use async_std::net::UdpSocket;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::sleep;

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

async fn dos_task(mut dos_socket: UdpSocket) {
    let mut buffer = [0; 1024];
    let mut active_users: Vec<User> = Vec::new();

    loop {
        if let Ok((_bytes_received, client_address)) = dos_socket.recv_from(&mut buffer).await {
            let message = String::from_utf8_lossy(&buffer[.._bytes_received]);

            if message.starts_with("REGISTER:") {
                // Extract username and user type from message
                let parts: Vec<&str> = message.trim().split(':').collect();
                let username = parts[1].trim();
                let user_type = parts[2].trim();
                let user_address: SocketAddr = client_address;
                let user_s_address=extract_ip_address(user_address);
                let new_user = User {
                    address: user_s_address,
                    name: username.to_string(),
                    user_type: user_type.to_string(),
                };
                active_users.push(new_user);
                println!("User registered: {} ({})", username, user_type);
                let response = "User registered";
                dos_socket
                    .send_to(response.as_bytes(), client_address)
                    .await
                    .expect("Failed to send response");
            } else if message.starts_with("UNREGISTER") {
                // Extract user's address and unregister
                let user_address: SocketAddr = client_address;
                let user_s_address=extract_ip_address(user_address);
                if let Some(index) = active_users.iter().position(|user| user.address == user_s_address) {
                    active_users.remove(index);
                    println!("User unregistered: {}", user_s_address);
                }
                let response = "User unregistered";
                dos_socket
                    .send_to(response.as_bytes(), client_address)
                    .await
                    .expect("Failed to send response");
            } else if message == "QUERY" {
                // Send the list of active users to the requesting client
                let users_list: String = active_users
                    .iter()
                    .map(|user| user.name.clone())
                    .collect::<Vec<String>>()
                    .join(",");
                dos_socket
                    .send_to(users_list.as_bytes(), client_address)
                    .await
                    .expect("Failed to send active users list");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("A");
    let dos_address: SocketAddr = "127.0.0.255:12345"
        .parse()
        .expect("Failed to parse dos address");
    let dos_socket = UdpSocket::bind(&dos_address)
        .await
        .expect("Failed to bind dos socket");

    let dos = dos_task(dos_socket);
    let _ = tokio::join!(dos);
}
