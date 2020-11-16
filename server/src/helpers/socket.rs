use pusher::PusherBuilder;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    value: String,
    user: String,
}

pub async fn pusher_message<V>(channel: &String, event: &String, message: &Message) {
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
