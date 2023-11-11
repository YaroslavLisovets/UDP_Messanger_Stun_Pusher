
use sha2::Sha256;
use hmac::{Hmac, Mac};
use hex::encode;
use serde_json::json;
use tokio_tungstenite::tungstenite::protocol::Message;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_auth_key(socket_id: &str, channel_name: &str, user_data: &str) -> String {
    let subject = format!(r"{}:{}:{}", socket_id, channel_name, user_data);
    let mut hmac = HmacSha256::new_from_slice(env!("pusher_secret").as_ref()).unwrap();
    hmac.update(subject.as_bytes());
    let result = hmac.finalize();
    let auth_key = format!("{}:{:}", env!("pusher_key"), encode(result.into_bytes()));
    auth_key
}

pub fn generate_auth_message(socket_id: &str) -> Message {
    let channel_name = "presence-channel";
    let user_data = r#"{"user_id": "159.89.173.104"}"#;
    let auth_key = generate_auth_key(socket_id, channel_name, user_data);
    Message::Text(json!({
        "event": "pusher:subscribe",
        "data": {
            "channel": channel_name,
            "auth": auth_key,
            "channel_data": user_data
        }
    }).to_string())
}


