use redis::{Client, Commands, RedisResult};
use uuid::Uuid;

use crate::models::{MessageFromOrderbook, MessageToEngine};

pub struct RedisManager {
    client: Client,
    publisher: Client,
}

impl RedisManager {
    pub fn new() -> Self {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        let publisher = redis::Client::open("redis://127.0.0.1/").unwrap();

        RedisManager { client, publisher }
    }

    pub async fn send_and_await(
        &self,
        message: MessageToEngine,
    ) -> RedisResult<MessageFromOrderbook> {
        let mut conn = self.client.get_connection().unwrap();
        let mut pub_conn = self.publisher.get_connection().unwrap();

        let client_id = Uuid::new_v4().to_string();

        let message_with_id = serde_json::json!({
            "clientId": client_id,
            "message": message
        });

        pub_conn.publish("messages", serde_json::to_string(&message_with_id).unwrap())?;

        // Subscribe and wait for response
        let mut pubsub = conn.as_pubsub();
        pubsub.subscribe(&client_id).unwrap();

        // Use recv() instead of get_message()
        let msg = pubsub.get_message().unwrap();

        let response: String = msg.get_payload()?;
        let parsed_response: MessageFromOrderbook = serde_json::from_str(&response).unwrap();

        Ok(parsed_response)
    }
}
