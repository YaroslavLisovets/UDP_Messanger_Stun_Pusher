extern crate pusher;
extern crate tokio;

mod stun;
mod pusher_interaction;

extern crate rand;
extern crate hyper;

use std::io::ErrorKind;
use std::net::{Ipv4Addr, UdpSocket};
use std::thread::sleep;
use std::time::Duration;
use pusher::Pusher;
use hyper::client::HttpConnector;
use crate::pusher_interaction::{get_pusher_client, get_remote_ip_without_waiting, get_remote_machine_ip};

enum ClientState {
    Connecting,
    Connected,
}

impl Default for ClientState {
    fn default() -> Self {
        ClientState::Connecting
    }
}

#[repr(u8)]
enum MessageType {
    TextMessage,
    // UpdateConnection,
    // Sync,
    Close,
    Connect,
}

#[tokio::main]
async fn main() {
    let uuid: u16 = rand::random();
    let pusher: Pusher<HttpConnector> = get_pusher_client();
    // SimpleLogger::new().with_colors(true).without_timestamps()1.init().unwrap();

    let (addr, remote_addr) = {
        if pusher.channel_users("presence-channel").await.unwrap().users.len() >= 1 {
            println!("Connecting to another user");
            get_remote_ip_without_waiting(uuid, pusher)
        } else {
            println!("Waiting for connection");
            get_remote_machine_ip(uuid, pusher)
        }
    };
    let mut remote_addr = remote_addr[3..].to_string();
    remote_addr.drain(..3);

    println!("{:?}", remote_addr.chars());
    println!("Connecting");
    println!("{}", remote_addr);
    hole_punch(&addr, &remote_addr);
    println!("Connected");
    communicate(&addr, &remote_addr);
}

fn hole_punch(addr: &(Ipv4Addr, u16, UdpSocket), remote_addr: &String) {
    let mut current_state = ClientState::Connecting;
    let socket = &addr.2;
    loop {
        socket.send_to(&[MessageType::Connect as u8], &remote_addr).expect("Failed to send message");
        let mut response = [0u8; 8];
        match socket.recv_from(&mut response) {
            Ok(_) => {
                if response[0] == MessageType::Connect as u8 {
                    socket.send_to([MessageType::Connect as u8].as_ref(), &remote_addr).expect("Failed to send message");
                    current_state = ClientState::Connected;
                    return;
                }
            }
            Err(err) => {
                if err.kind() == ErrorKind::TimedOut {
                    println!("Communication with remote machine timed out.");
                } else {
                    eprintln!("Error receiving response: {}", err);
                }
            }
        }
        sleep(Duration::from_millis(500));
        println!("Waiting");
    }
}

fn communicate(addr: &(Ipv4Addr, u16, UdpSocket), remote_addr: &String) {
    loop {
        let socket = &addr.2;
        let message = "Don't jerk off ";
        socket.send_to(message.as_bytes(), &remote_addr).expect("Failed to send message");
        let mut response = [0u8; 256];
        match socket.recv_from(&mut response) {
            Ok((size, remote)) => {
                let response_str = String::from_utf8_lossy(&response[..size]);
                println!("Received response from remote machine at {:?}: {}", remote, response_str);
                if response[0] == MessageType::TextMessage as u8 {
                    println!("Get message");
                }
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


