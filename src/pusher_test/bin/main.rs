mod auth_message_gen;
const CONNECTION: &str = "wss://ws-eu.pusher.com:443/app/5bca62ed4d7914057704?version=1.0.7&protocol=6";

use futures_util::{future, pin_mut, StreamExt};
use tokio_tungstenite::{connect_async};
use crate::auth_message_gen::generate_auth_message;


#[tokio::main]
async fn main() {
    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();


    let (ws_stream, _) = connect_async(CONNECTION).await.expect("Failed to connect");
    let (write, mut read) = ws_stream.split();
    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let message = read.next().await.unwrap();
    let message = message.unwrap().into_text().unwrap();
    let json: serde_json::Value = serde_json::from_str(message.as_str()).unwrap();
    let socket_id: serde_json::Value = serde_json::from_str(json["data"].as_str().unwrap()).unwrap();
    let socket_id = socket_id["socket_id"].as_str().unwrap();

    stdin_tx.unbounded_send(generate_auth_message(socket_id)).unwrap();
    


    let ws_to_stdout = {
        read.for_each(|message| async {
            let message = message.unwrap();
            println!("{}", message);
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

