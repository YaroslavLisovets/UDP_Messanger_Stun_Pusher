pub extern crate futures;


use std::net::{Ipv4Addr, TcpStream, UdpSocket};
use std::thread::sleep;
use std::time::Duration;
use futures::executor;
use hyper::client::HttpConnector;
use pusher::{Pusher, PusherBuilder};
use serde_json::{json, Value};
use websocket::{ClientBuilder, OwnedMessage};
use websocket::receiver::Reader;
use websocket::sender::Writer;
use crate::pusher_interaction::auth_message_gen::{generate_auth_key, PusherType};
use crate::stun::resolve_udp_addr_v4;

pub mod auth_message_gen;


pub fn generate_auth_message(socket_id: &str, uuid:u16, pusher_type: Option<PusherType>) -> OwnedMessage {
    let pusher_type:String = pusher_type.unwrap_or(PusherType::Subscribe).into();
    let channel_name = "presence-channel";
    let user_data = format!("{{\"user_id\": {uuid}}}");
    let auth_key = generate_auth_key(socket_id, channel_name, user_data.as_str());

    OwnedMessage::Text(json!({
        "event": pusher_type,
        "data": {
            "channel": channel_name,
            "auth": auth_key,
            "channel_data": user_data
        }
    }).to_string())
}


pub fn wait_for_event<F>(event_handler: F, uuid: u16) -> String where F: Fn(String) -> bool, {
    let (mut receiver, mut sender) = connect_client(uuid);
    loop {
        let message = receiver.incoming_messages().next().unwrap();
        let message = match message {
            Ok(m) => m,
            Err(err) => {
                let _ = sender.send_message(&OwnedMessage::Close(None));
                // sender.send_message(
                //     &generate_auth_message("", uuid, Some(Unsubscribe))).unwrap();
                panic!("Receive Error: {}", err);
            }
        };
        match message {
            OwnedMessage::Text(text) => {
                let json: Value = serde_json::from_str(text.as_str()).unwrap();
                if let Value::String(event) = json["event"].clone() {
                    if event_handler(event.as_str().parse().unwrap()) {
                        sender.send_message(&OwnedMessage::Close(None)).unwrap();
                        return json["data"].to_string();
                    }
                }
            }
            OwnedMessage::Ping(data) => {
                sender.send_message(&OwnedMessage::Pong(data)).unwrap();
            }
            _ => {}
        };
    }
}

pub fn get_remote_ip_without_waiting(uuid: u16, pusher: Pusher<HttpConnector>) -> ((Ipv4Addr, u16, UdpSocket), String) {
    let addr = match resolve_udp_addr_v4(None) {
        Ok(ip) => {
            ip
        }
        Err(message) => {
            panic!("Couldn't Resolve Udp Address {:?} ", message);
        }
    };
    let addr_str = addr.0.to_string() + ":" + addr.1.to_string().as_str();
    executor::block_on(pusher.trigger("presence-channel", "connect", addr_str)).unwrap();
    let data = wait_for_event(move |event_name| event_name == "answer", uuid);
    (addr, data)
}

pub fn get_remote_machine_ip(uuid: u16, pusher: Pusher<HttpConnector>) -> ((Ipv4Addr, u16, UdpSocket), String) {
    let data = wait_for_event(|event_name| event_name == "connect", uuid);
    let addr = match resolve_udp_addr_v4(None) {
        Ok(ip) => {
            ip
        }
        Err(message) => {
            panic!("Couldn't Resolve Udp Address {:?} ", message);
        }
    };
    let addr_str = addr.0.to_string() + ":" + addr.1.to_string().as_str();
    sleep(Duration::from_millis(500));
    executor::block_on(pusher.trigger("presence-channel", "answer", addr_str)).unwrap();
    (addr, data)
}

pub fn get_pusher_client() -> Pusher<HttpConnector> {
    let pusher_app_id = env!("pusher_app_id");
    let pusher_key = env!("pusher_key");
    let pusher_secret = env!("pusher_secret");

    let mut _pusher: Pusher<HttpConnector> = PusherBuilder::new(pusher_app_id, pusher_key, pusher_secret).finalize();
    _pusher.host = "api-eu.pusher.com".to_string();
    _pusher
}

const CONNECTION: &str = "ws://ws-eu.pusher.com:80/app/5bca62ed4d7914057704?version=1.0.7&protocol=6";


fn connect_client(uuid: u16) -> (Reader<TcpStream>, Writer<TcpStream>) {
    let client = ClientBuilder::new(CONNECTION)
        .unwrap()
        .connect_insecure()
        .unwrap();
    let (mut receiver, mut sender) = client.split().unwrap();
    let message = receiver.recv_message().unwrap();
    if let OwnedMessage::Text(text) = message {
        let json: Value = serde_json::from_str(text.as_str()).unwrap();
        let socket_id: Value = serde_json::from_str(json["data"].as_str().unwrap()).unwrap();
        let socket_id = socket_id["socket_id"].as_str().unwrap();
        sender.send_message(&generate_auth_message(socket_id, uuid, None)).unwrap();
    };
    (receiver, sender)
}



