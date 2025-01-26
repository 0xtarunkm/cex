use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub subscriptions: Arc<Mutex<Vec<String>>>,
}

impl User {
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            subscriptions: Arc::new(Mutex::new(Vec::new())),
        }
    }
}
