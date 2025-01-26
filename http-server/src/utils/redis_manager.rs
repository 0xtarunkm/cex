use redis::{Client, Commands, RedisResult};
use uuid::Uuid;

use crate::models::MessageToEngine;

pub struct RedisManager {
    client: Client,
}

impl RedisManager {
    pub fn new() -> Self {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        RedisManager { client }
    }

    pub async fn send_and_await(&self, message: MessageToEngine) -> RedisResult<String> {
        let mut conn = self.client.get_connection().unwrap();

        let client_id = Uuid::new_v4().to_string();

        let message_with_id = serde_json::json!({
            "client_id": client_id,
            "message": message
        });

        let _: () = conn.lpush("messages", serde_json::to_string(&message_with_id).unwrap())?;

        let mut pubsub = conn.as_pubsub();
        pubsub.subscribe(&client_id).unwrap();

        let msg = pubsub.get_message().unwrap();
        let response: String = msg.get_payload()?;

        let parsed_response = serde_json::from_str(&response).unwrap();

        Ok(parsed_response)
    }
}
