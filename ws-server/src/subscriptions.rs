use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct SubscriptionManager {
    subscriptions: Arc<RwLock<HashMap<String, Vec<Uuid>>>>,
    redis_client: redis::Client,
}

impl SubscriptionManager {
    pub async fn new() -> Self {
        let redis_client =
            redis::Client::open("redis://127.0.0.1/").expect("Invalid Redis connection");

        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            redis_client,
        }
    }

    pub async fn subscribe(&self, user_id: Uuid, topic: String) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions
            .entry(topic.clone())
            .or_insert_with(Vec::new)
            .push(user_id);
    }

    pub async fn unsubscribe(&self, user_id: Uuid, topic: &str) {
        let mut subscriptions = self.subscriptions.write().await;
        if let Some(users) = subscriptions.get_mut(topic) {
            users.retain(|&id| id != user_id);
        }
    }
}
