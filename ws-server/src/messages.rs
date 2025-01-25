use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum IncomingMethod {
    Subscribe,
    Unsubscribe,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IncomingMessage {
    pub method: IncomingMethod,
    pub params: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutgoingMessage {
    pub topic: String,
    pub data: serde_json::Value,
}
