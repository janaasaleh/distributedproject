use async_std::net::UdpSocket;
use std::net::SocketAddr;
use std::time::Duration;
use rand::Rng;
use std::thread;
use std::time;
use tokio::time::sleep;
use tokio::time::Instant;

async fn server3(server_address: &str, _middleware_address: &str) {
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
    println!("Server 3 socket is listening on {}", server_address);

    let mut buffer = [0; 1024];

    while let Ok((_bytes_received, client_address)) = socket.recv_from(&mut buffer).await {
        let message = String::from_utf8_lossy(&buffer);
        println!("Server 3 received: {}", message);

        let response = match port {
            54322 => "Server 3 received your message",
            _ => "Server 3 received your message",
        };

        println!("Server 3 responding with: {}", response);
        //sleep(Duration::from_millis(10000)).await;

        // Send the response to the client's middleware
        if let Err(err) = socket.send_to(response.as_bytes(), client_address).await {
            eprintln!(
                "Server 3 failed to send acknowledgment to middleware: {}",
                err
            );
        }
        println!("Middleware address {}", client_address);
        // Clear the buffer for the next request
        buffer = [0; 1024];
    }
}

async fn server_middleware(middleware_address: &str, server_addresses: Vec<&str>) {
    let middleware_socket = UdpSocket::bind(middleware_address)
        .await
        .expect("Failed to bind middleware socket");
    let server_to_server_socket = UdpSocket::bind("127.0.0.4:8080")
        .await
        .expect("Failed to bind server to server socket");
    let just_up_socket = UdpSocket::bind("127.0.0.4:8087")
        .await
        .expect("Failed to bind server to server socket");
    let skip_socket = UdpSocket::bind("127.0.0.4:8088")
        .await
        .expect("Failed to bind server to server socket");


    let mut server_down=0;
    let mut real_server_down=0;
    let mut server_down_index:i32=12;
    let mut server_up_index:i32=22;
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
    while let Ok((bytes_received, client_address)) = middleware_socket.recv_from(&mut receive_buffer).await
    {
        println!("Just Slept:{}",just_slept);
        //println!("{:?}",receive_buffer);
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

        if(which_server==0)
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

        let mut server_to_server1_receive_buffer = [0; 4];
        let mut server_to_server2_receive_buffer = [0; 4];
        let mut just_up_receive_buffer = [0; 4];


        //server_to_server_socket
        //.connect("127.0.0.2:8080")
        //.await
        //.expect("Failed to connect to the server");
//
        //server_to_server_socket
        //.connect("127.0.0.3:8080")
        //.await
        //.expect("Failed to connect to the server");
        if random_number < 3 && server_down==0 && real_server_down==0 && previous_down==0{
        server_down=0;
        own_down=1;
        server_to_server_socket
            .send_to(down_index, "127.0.0.2:8080")
            .await
            .expect("Failed to send index to server 1");
        server_to_server_socket
            .send_to(down_index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 2");
        println!("Server 3 going down!");
        }
        else {
            if(which_server==500)
            {
            server_to_server_socket
            .send_to(index, "127.0.0.2:8080")
            .await
            .expect("Failed to send index to server 1");
            server_to_server_socket
            .send_to(index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 2");
            }
            else if which_server==0
            {
                server_to_server_socket
            .send_to(index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 2");
            }
            else if which_server==1
            {
                server_to_server_socket
            .send_to(index, "127.0.0.2:8080")
            .await
            .expect("Failed to send index to server 1");
            }
        }

        if previous_down>0
        {
            previous_down=previous_down-1;
        }


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

        if(server_down==0 || index1>19)
        {
        server_to_server_socket
            .recv_from(&mut server_to_server2_receive_buffer)
            .await
            .expect("Couldn't recieve index");
        }


        let mut index2 = i32::from_be_bytes([
            server_to_server2_receive_buffer[0],
            server_to_server2_receive_buffer[1],
            server_to_server2_receive_buffer[2],
            server_to_server2_receive_buffer[3],
        ]);

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
                if(index1 == 10)
                {
                    println!("Server 1 is down");
                    server_down=1;
                    which_server=0;
                }
                if(index1 == 11)
                {
                    println!("Server 2 is down");
                    server_down=1;
                    which_server=1;
                }

            }
            else {
                if index1==20
                {
                    println!("Server 1 is back up");
                    server_down=0;
                    which_server=500;
                    just_up=1;
                }
                if index1==21
                {
                    println!("Server 2 is back up");
                    server_down=0;
                    which_server=500;
                }
               
            }

        }
        if (index2>9)
        {
            if index2<19
            {
                if(index2 == 10)
                {
                    println!("Server 1 is down");
                    server_down=1;
                    which_server=0;
                }
                if(index2 == 11)
                {
                    println!("Server 2 is down");
                    server_down=1;
                    which_server=1;
                }

            }
            else {
                if index2==20
                {
                    println!("Server 1 is back up");
                    server_down=0;
                    which_server=500;
                    just_up=1;
                }
                if index2==21
                {
                    println!("Server 2 is back up");
                    server_down=0;
                    which_server=500;
                }
               
            }

        }

        if(which_server==500)
        {
        if current_server == 0{
            current_server += 1;
            if(just_up==1)
            {
            let cs: &[u8] = &current_server.to_be_bytes();
            server_to_server_socket
            .send_to(cs, "127.0.0.2:8087")
            .await.expect("Failed to send index to server 1");
            let sdr: &[u8] = &server_down_requests.to_be_bytes();
            server_to_server_socket
            .send_to(sdr, "127.0.0.2:8088")
            .await.expect("Failed to send index to server 3");
            just_up=0;
            }
            if(own_down==1)
            {
                thread::sleep(time::Duration::from_secs(40));
            previous_down=6;
            println!("Server 3 going up!");
            server_to_server_socket
            .send_to(up_index, "127.0.0.2:8080")
            .await
            .expect("Failed to send index to server 1");
            server_to_server_socket
            .send_to(up_index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 2");
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
            println!("{:?}",receive_buffer);
            num_down+=1;
            if(num_down>1)
            {
                //just_slept=just_slept+1;
                just_slept=just_slept;
            }
            }
            continue;
        }
        else if current_server == 1{
            current_server += 1;
            if(just_up==1)
            {
            let cs: &[u8] = &current_server.to_be_bytes();
            server_to_server_socket
            .send_to(cs, "127.0.0.2:8087")
            .await.expect("Failed to send index to server 1");
            let sdr: &[u8] = &server_down_requests.to_be_bytes();
            server_to_server_socket
            .send_to(sdr, "127.0.0.2:8088")
            .await.expect("Failed to send index to server 3");
            just_up=0;
            }
            if(own_down==1)
            {
                thread::sleep(time::Duration::from_secs(40));
            previous_down=6;
            println!("Server 3 going up!");
            server_to_server_socket
            .send_to(up_index, "127.0.0.2:8080")
            .await
            .expect("Failed to send index to server 1");
            server_to_server_socket
            .send_to(up_index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 2");
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
            println!("Current Server Down {}",current_server);
            println!("I am here 2");
            just_up_receive_buffer = [0; 4];
            //receive_buffer = [0; 1024];
            //receive_buffer = [0; 1024];
            println!("{:?}",receive_buffer);
            num_down+=1;
            if(num_down>1)
            {
                //just_slept=just_slept+1;
                just_slept=just_slept;
            }
            }
            continue;
        } else if current_server == 2{
            current_server = 0;
            if(just_up==1)
            {
            println!("I am here");    
            let cs: &[u8] = &current_server.to_be_bytes();
            server_to_server_socket
            .send_to(cs, "127.0.0.2:8087")
            .await.expect("Failed to send index to server 1");
            let sdr: &[u8] = &server_down_requests.to_be_bytes();
            server_to_server_socket
            .send_to(sdr, "127.0.0.2:8088")
            .await.expect("Failed to send index to server 3");
            just_up=0;
            }
           
        }
    }
    else if (which_server==0)
    {

        if current_server == 0 {
            current_server += 1;
            continue;
        } else if current_server == 1{
            current_server += 1;
            continue;
        } else if current_server == 2{
            current_server = 1;
        }
    }
    else if (which_server==1)
    {
        if current_server == 0 && which_server==1{
            current_server += 2;
            continue;
        } else if current_server == 1 && which_server==1{
            current_server += 1;
            continue;
        } else if current_server == 2 && which_server==1{
            current_server = 0;
        }
    }
    else {
        println!("No which server variable");
    }

        let server_index = 2; // You can implement load balancing logic here
        let server_address = server_addresses[server_index];
        let server_address: SocketAddr = server_address
            .parse()
            .expect("Failed to parse server address");

        let server_socket = UdpSocket::bind("127.0.0.4:0")
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
        println!("Client Address:{}",client_address);

        // Clear the receive buffer for the next request
        server_to_server1_receive_buffer = [0; 4];
        server_to_server2_receive_buffer = [0; 4];
        receive_buffer = [0; 1024];
        if(own_down==1)
        {
            thread::sleep(time::Duration::from_secs(40));
            previous_down=6;
            println!("Server 3 going up!");
            server_to_server_socket
            .send_to(up_index, "127.0.0.2:8080")
            .await
            .expect("Failed to send index to server 1");
            server_to_server_socket
            .send_to(up_index, "127.0.0.3:8080")
            .await
            .expect("Failed to send index to server 2");
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
            just_up_receive_buffer = [0; 4];
            if(just_slept>90)
            {
                just_slept=93;
            }
            //receive_buffer = [0; 1024];
            //receive_buffer = [0; 1024];
            println!("{:?}",receive_buffer);
            num_down+=1;
            if(num_down>1)
            {
                //just_slept=just_slept+1;
                just_slept=just_slept;
            }
            println!("Current Server Down {}",current_server);
            println!("I am here 3");

        }
    }
}

#[tokio::main]
async fn main() {
    let middleware_address: SocketAddr = "127.0.0.4:21113"
        .parse()
        .expect("Failed to parse middleware address");
    let middleware_address_str = middleware_address.to_string();

    // Define the server addresses and middleware addresses
    let server_addresses = ["127.0.0.2:54321", "127.0.0.3:54322", "127.0.0.4:54323"];
    let server3_task = server3("127.0.0.4:54323", &middleware_address_str);

    // Start the server middleware
    let server_middleware_task =
        server_middleware(&middleware_address_str, server_addresses.to_vec());
    let _ = tokio::join!(server3_task, server_middleware_task);
}