extern crate pusher;
extern crate tokio;

mod stun;

extern crate rand;


use std::io;
use std::io::ErrorKind;
use pusher::{ChannelUser, PusherBuilder};
use stun::resolve_udp_addr_v4;

enum State {
    WaitingForEndpoint,
    Connected,
}

enum MessageType {
    TextMessage,
    UpdateConnection,
    Sync,
    Close,
}

#[tokio::main]
async fn main() {
    let addr = match resolve_udp_addr_v4(None) {
        Ok(ip) => {
            ip
        }
        Err(message) => {
            println!("Couldn't Resolve Udp Address {:?} ", message);
            return;
        }
    };
    println!("{:?}", addr);

    println!("Enter the remote machine's IP address and port (e.g., 192.168.0.2:12345): ");
    let mut remote_machine_input = String::new();
    io::stdin().read_line(&mut remote_machine_input).expect("Failed to read input");

    let parts: Vec<&str> = remote_machine_input.trim().split(':').collect();

    if parts.len() != 2 {
        println!("Invalid input. Please enter the IP address and port in the format 'IP:Port'.");
        return;
    }

    let remote_addr = remote_machine_input;//format!("{}:{}", remote_ip, remote_port);

    loop {
        let socket = &addr.2;
        let message = "Don't jerk off ";
        socket.send_to(message.as_bytes(), &remote_addr).expect("Failed to send message");

        let mut response = [0u8; 1024];
        match socket.recv_from(&mut response) {
            Ok((size, remote)) => {
                let response_str = String::from_utf8_lossy(&response[..size]);
                println!("Received response from remote machine at {:?}: {}", remote, response_str);
            }
            Err(err) => {
                if err.kind() == ErrorKind::TimedOut {
                    println!("Communication with remote machine timed out.");
                } else {
                    eprintln!("Error receiving response: {}", err);
                }
            }
        }
    }
}


async fn get_pusher_users() -> Vec<ChannelUser> {
    let pusher_app_id = env!("pusher_app_id");
    let pusher_key = env!("pusher_key");
    let pusher_secret = env!("pusher_secret");
    // let pusher_cluster = env!("pusher_cluster");

    let mut pusher = PusherBuilder::new(pusher_app_id, pusher_key, pusher_secret).finalize();
    pusher.host = "api-eu.pusher.com".to_string();

    // println!("{:?}", );
    pusher.channel_users("presence-channel").await.unwrap().users
}

