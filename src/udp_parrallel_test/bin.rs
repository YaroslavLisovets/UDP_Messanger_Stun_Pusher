use std::net::Ipv4Addr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::UdpSocket;
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).await.unwrap();
    socket.connect("127.0.0.1:8080").await.unwrap();
    let r = Arc::new(socket);
    let s = r.clone();
    tokio::spawn(async move {
        let message = "2".as_bytes();
        loop {
            println!("1");
            r.send(message).await.unwrap();
        };
    });
    let message = "1".as_bytes();
    loop {
        s.send(message).await.unwrap();
    }
}