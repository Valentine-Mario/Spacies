use crate::handlers::types::*;
use pusher::PusherBuilder;

pub async fn push_user_message(channel: &String, event: &String, message: &UserMessage) {
    let pusher_url = format!(
        "http://{}:{}@api-{}.pusher.com/apps/{}",
        std::env::var("PUSHER_KEY").expect("PUSHER KEY not set"),
        std::env::var("PUSHER_SECRET").expect("PUSHER SECRET not set"),
        std::env::var("PUSHER_CLUSTER").expect("PUSHER CLUSTER not set"),
        std::env::var("PUSHER_ID").expect("PUSHER ID not set"),
    );

    let pusher = PusherBuilder::from_url(&pusher_url).finalize();
    pusher.trigger(channel, event, message).await.unwrap();
}

pub async fn push_channel_message(channel: &String, event: &String, message: &ChannelMessage) {
    let pusher_url = format!(
        "http://{}:{}@api-{}.pusher.com/apps/{}",
        std::env::var("PUSHER_KEY").expect("PUSHER KEY not set"),
        std::env::var("PUSHER_SECRET").expect("PUSHER SECRET not set"),
        std::env::var("PUSHER_CLUSTER").expect("PUSHER CLUSTER not set"),
        std::env::var("PUSHER_ID").expect("PUSHER ID not set"),
    );

    let pusher = PusherBuilder::from_url(&pusher_url).finalize();
    pusher.trigger(channel, event, message).await.unwrap();
}
