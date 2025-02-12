use lazy_static::lazy_static;
use redis::{Client, RedisResult};

#[allow(deprecated)]
use redis::aio::Connection as AsyncConnection;

lazy_static! {
    static ref REDIS_MANAGER: RedisManager = RedisManager::new();
}

pub struct RedisManager {
    client: Client,
}

impl RedisManager {
    fn new() -> Self {
        let client = redis::Client::open("redis://redis:6379").unwrap();
        RedisManager { client }
    }

    pub fn instance() -> &'static RedisManager {
        &REDIS_MANAGER
    }

    #[allow(deprecated)]
    pub async fn get_async_connection(&self) -> RedisResult<AsyncConnection> {
        self.client.get_async_connection().await
    }
}
