use sha2::Sha256;
use hmac::{Hmac, Mac};
use hex::encode;



type HmacSha256 = Hmac<Sha256>;

pub fn generate_auth_key(socket_id: &str, channel_name: &str, user_data: &str) -> String {
    let subject = format!(r"{}:{}:{}", socket_id, channel_name, user_data);
    let mut hmac = HmacSha256::new_from_slice(env!("pusher_secret").as_ref()).unwrap();
    hmac.update(subject.as_bytes());
    let result = hmac.finalize();
    let auth_key = format!("{}:{:}", env!("pusher_key"), encode(result.into_bytes()));
    auth_key
}

pub enum PusherType{
    Subscribe,
    Unsubscribe
}
impl From<PusherType> for String{
    fn from(value: PusherType) -> Self {
        match value {
            PusherType::Subscribe => { "pusher:subscribe" }
            PusherType::Unsubscribe => { "pusher:unsubscribe" }
        }.to_string()
    }
}



