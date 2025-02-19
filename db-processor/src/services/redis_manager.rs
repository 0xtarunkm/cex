use lazy_static::lazy_static;

use redis::{Client, Connection, RedisResult};

lazy_static! {
    static ref REDIS_MANAGER: RedisManager = RedisManager::new();
}

pub struct RedisManager {
    client: Client,
}

impl RedisManager {
    fn new() -> Self {
        let client = redis::Client::open(std::env::var("REDIS_URL").unwrap()).unwrap();
        RedisManager { client }
    }

    pub fn instance() -> &'static RedisManager {
        &REDIS_MANAGER
    }

    pub fn get_connection(&self) -> RedisResult<Connection> {
        self.client.get_connection()
    }
}
