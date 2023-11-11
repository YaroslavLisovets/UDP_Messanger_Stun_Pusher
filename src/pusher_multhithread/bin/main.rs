use pusher::{PusherBuilder};

use std::env;

#[tokio::main]
async fn main() {
    let pusher_app_id = env!("pusher_app_id");
    let pusher_key = env!("pusher_key");
    let pusher_secret = env!("pusher_secret");
    // let pusher_cluster = env!("pusher_cluster");

    let pusher = PusherBuilder::new(pusher_app_id, pusher_key, pusher_secret);
    // pusher.host = pusher_cluster.to_string();

    let mut pusher = pusher.finalize();
    // println!("{}", pusher.host);
    pusher.host = "api-eu.pusher.com".to_string();
    let channels_params = vec![("filter_by_prefix".to_string(), "presence-".to_string()), ("info".to_string(), "user_count".to_string())];
    println!("{:?}", pusher.channels_with_options(channels_params).await);
    // println!("{:?}", );
    let users =  pusher.channel_users("presence-channel").await.unwrap().users;

    // let result = pusher.trigger("test_channel", "announce", "hello world!").await;

    // match result {
    //     Ok(events) => println!("Successfully published: {:?}", events),
    //     Err(err) => println!("Failed to publish: {}", err),
    // }
}