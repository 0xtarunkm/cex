use redis::{Client, Commands, RedisResult};
use tracing::info;
use uuid::Uuid;

use crate::models::{MessageFromEngine, MessageToEngine, OnRampPayload};

pub struct RedisManager {
    client: Client,
}

impl RedisManager {
    pub fn new() -> Self {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        RedisManager { client }
    }

    pub fn send_and_wait(&self, message: MessageToEngine) -> RedisResult<MessageFromEngine> {
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

        let parsed_response: MessageFromEngine = serde_json::from_str(&response).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::ParseError,
                "Failed to parse engine response",
                e.to_string(),
            ))
        })?;

        Ok(parsed_response)
    }

    pub fn onramp_and_wait(&self, message: OnRampPayload) -> RedisResult<String> {
        let mut conn = self.client.get_connection().unwrap();
        let client_id = Uuid::new_v4().to_string();

        let message_with_id = serde_json::json!({
            "client_id": client_id,
            "message": message
        });

        let _: () = conn.lpush("onramp", serde_json::to_string(&message_with_id).unwrap())?;

        let mut pubsub = conn.as_pubsub();
        pubsub.subscribe(&client_id).unwrap();
        info!("Onramp response: cool");

        let msg = pubsub.get_message().unwrap();
        let response: String = msg.get_payload()?;

        info!("Onramp response: {}", response);

        Ok(response)
    }
}
