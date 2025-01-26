use crate::trade::Fill;
use lazy_static::lazy_static;
use redis::{Client, Commands, RedisError};
use serde_json;
use std::sync::Mutex;

pub struct RedisManager {
    client: Client,
}

impl RedisManager {
    fn new() -> Self {
        let client = redis::Client::open("redis://127.0.0.1/").unwrap();
        RedisManager { client }
    }

    pub fn send_to_api(
        &self,
        order_id: String,
        executed_qty: f64,
        client_id: String,
    ) -> Result<(), RedisError> {
        let mut conn = self.client.get_connection()?;

        let message_with_id = serde_json::json!({
            "client_id": client_id,
            "order_id": order_id,
            "executed_qty": executed_qty
        });

        let _: () = conn.publish(client_id, serde_json::to_string(&message_with_id).unwrap())?;

        Ok(())
    }
}

lazy_static! {
    static ref REDIS_MANAGER: Mutex<RedisManager> = Mutex::new(RedisManager::new());
}

pub fn get_redis_manager() -> &'static Mutex<RedisManager> {
    &REDIS_MANAGER
}
