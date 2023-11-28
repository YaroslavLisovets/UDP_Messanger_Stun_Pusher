extern crate tungstenite;

use tokio::net::UdpSocket;
use std::io;
use tungstenite::{connect, Message};

#[tokio::main]
async fn main() {
    let sock = UdpSocket::bind("0.0.0.0:8080").await.unwrap();
    let mut buf = [0; 1024];
    // let mut map = HashMap::new();


    loop {
        let (len, addr) = sock.recv_from(&mut buf).await.unwrap();
        let response_str = String::from_utf8_lossy(&buf[..len]);
        println!("{}: {}", addr, response_str);
        // println!()
        // sock.send_to(&buf[..len], addr).await.unwrap();
        // *map.entry(len).or_insert(0u16) += 1;
    }
    // println!("{:?}", map);
}