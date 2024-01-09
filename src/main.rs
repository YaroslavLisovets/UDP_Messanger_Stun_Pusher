extern crate pusher;
extern crate tokio;


mod stun;
mod pusher_interaction;

extern crate rand;
extern crate hyper;

use std::{sync::Arc, sync::Mutex, net::Ipv4Addr, thread};
use std::io::{stdin, stdout, Write};

use std::time::Duration;
use pusher::Pusher;
use hyper::client::HttpConnector;
// use tokio::io;
// use tokio::io::{AsyncBufReadExt, AsyncWriteExt};


use std::net::UdpSocket;
use std::process::Command;
use std::thread::sleep;
// use tokio::time::sleep;
use crate::ClientState::{Connected, Connecting};
use crate::pusher_interaction::{get_pusher_client, get_remote_ip_without_waiting, get_remote_machine_ip};

#[derive(Default)]
enum ClientState {
    #[default]
    Connecting,
    Connected,
}


#[tokio::main]
async fn main() {
    let uuid: u16 = rand::random();
    let pusher: Pusher<HttpConnector> = get_pusher_client();

    let (addr, remote_addr) = {
        if !pusher.channel_users("presence-channel").await.unwrap().users.is_empty() {
            println!("Connecting to another user");
            get_remote_ip_without_waiting(uuid, pusher).await
        } else {
            println!("Waiting for connection");
            // std::process::Command::new("clear").status().unwrap();
            get_remote_machine_ip(uuid, pusher).await
        }
    };
    print!("\x1B[2J");
    let remote_addr = remote_addr[3..remote_addr.len() - 3].to_string();
    run_communication(addr, remote_addr);
}


fn run_communication(addr: (Ipv4Addr, u16, UdpSocket), remote_addr: String) {
    let r = Arc::new(addr.2);
    let s = r.clone();
    // let (mut message_counter, mut clients_counters) = (0u32, 0u32);
    r.connect(remote_addr).unwrap();
    let internal_state = Arc::new(Mutex::new(Connecting));
    let stdin = stdin();
    let internal_state_clone = Arc::clone(&internal_state);
    thread::spawn(move || {
        loop {
            let state = internal_state_clone.lock().unwrap();
            match *state {
                Connecting => {
                    r.send(&MessageType::Connect(addr.0, addr.1).as_bytes()).unwrap();
                    drop(state);
                    sleep(Duration::from_secs(1));
                }
                Connected => {
                    let mut input_string = String::new();
                    stdin.read_line(&mut input_string).unwrap();
                    r.send(&MessageType::TextMessage(input_string).as_bytes()).unwrap();
                }
            }
        }
    });
    sleep(Duration::from_secs(1));

    loop {
        let mut stdout = stdout();
        let mut buf = [0u8; 1024];
        match s.recv_from(&mut buf) {
            Ok((len, _)) => {
                match MessageType::from_bytes(&buf[..len]).unwrap() {
                    MessageType::TextMessage(message) => {
                        s.send_to(&buf[..len], "128.0.0.1:8080").unwrap();
                        stdout.write_all(format!("Received: {}\n", message).as_bytes()).unwrap();

                        print!("\x1B[2J");
                    }
                    MessageType::UpdateConnection(_, _) => {}
                    MessageType::Connect(_, _) => {
                        if let Connected = *internal_state.lock().unwrap() {
                            continue;
                        }
                        stdout.write_all("Connected\n".as_bytes()).unwrap();
                        *internal_state.lock().unwrap() = Connected;
                    }
                    MessageType::Close => {}
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}

#[repr(u8)]
enum MessageHeader {
    TextMessage,
    UpdateConnection,
    Close,
    Connect,
}

#[derive(Debug)]
enum MessageType {
    TextMessage(String),
    UpdateConnection(Ipv4Addr, u16),
    Connect(Ipv4Addr, u16),
    Close,
}

impl MessageType {
    fn as_bytes(&self) -> Vec<u8> {
        match self {
            MessageType::TextMessage(message) => {
                let mut bytes = Vec::with_capacity(message.len() + 1);
                bytes.push(MessageHeader::TextMessage as u8);
                for char in message.chars() {
                    bytes.push(char as u8);
                }
                bytes
            }
            MessageType::UpdateConnection(addr, socket) => {
                let mut bytes = Vec::with_capacity(23);
                bytes.push(MessageHeader::UpdateConnection as u8);
                bytes.extend_from_slice(&addr.octets());
                bytes.extend_from_slice(&socket.to_be_bytes());
                bytes
            }
            MessageType::Close => { vec![MessageHeader::Close as u8] }
            MessageType::Connect(addr, socket) => {
                let mut bytes = Vec::with_capacity(23);
                bytes.push(MessageHeader::Connect as u8);
                bytes.extend_from_slice(&addr.octets());
                bytes.extend_from_slice(&socket.to_be_bytes());
                bytes
            }
        }
    }
    fn from_bytes(bytes: &[u8]) -> Option<MessageType> {
        if let Some(&header) = bytes.first() {
            match header {
                x if x == MessageHeader::TextMessage as u8 => {
                    let text = String::from_utf8_lossy(&bytes[1..]).to_string();
                    Some(MessageType::TextMessage(text))
                }
                x if x == MessageHeader::UpdateConnection as u8 => {
                    if bytes.len() >= 7 {
                        let addr = Ipv4Addr::new(bytes[1], bytes[2], bytes[3], bytes[4]);
                        let socket = u16::from_be_bytes([bytes[5], bytes[6]]);
                        Some(MessageType::UpdateConnection(addr, socket))
                    } else {
                        None
                    }
                }
                x if x == MessageHeader::Close as u8 => Some(MessageType::Close),
                x if x == MessageHeader::Connect as u8 => {
                    if bytes.len() >= 7 {
                        let addr = Ipv4Addr::new(bytes[1], bytes[2], bytes[3], bytes[4]);
                        let socket = u16::from_be_bytes([bytes[5], bytes[6]]);
                        Some(MessageType::Connect(addr, socket))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}