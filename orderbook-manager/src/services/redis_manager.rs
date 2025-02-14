use lazy_static::lazy_static;

use redis::{Client, Commands, Connection, RedisResult};
use serde_json::Value;

use crate::models::MessageToApi;

lazy_static! {
    static ref REDIS_MANAGER: RedisManager = RedisManager::new();
}

pub struct RedisManager {
    client: Client,
}

impl RedisManager {
    fn new() -> Self {
        let client = redis::Client::open("redis://0.0.0.0:6379").unwrap();
        RedisManager { client }
    }

    pub fn instance() -> &'static RedisManager {
        &REDIS_MANAGER
    }

    pub fn get_connection(&self) -> RedisResult<Connection> {
        self.client.get_connection()
    }

    pub fn send_to_api(&self, client_id: &str, message: &MessageToApi) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        let message_json = serde_json::to_string(message).unwrap();
        conn.publish(client_id, message_json)
    }

    pub fn publish_message(&self, channel: &str, message: &Value) -> RedisResult<()> {
        let mut conn = self.get_connection()?;
        conn.publish(channel, message.to_string())
    }
}