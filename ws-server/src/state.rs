use crate::subscriptions::SubscriptionManager;
use crate::user::User;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<RwLock<HashMap<Uuid, User>>>,
    pub subscription_manager: Arc<SubscriptionManager>,
}
