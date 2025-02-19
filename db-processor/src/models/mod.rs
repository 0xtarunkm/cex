use message_from_engine::AddTradePayload;
use serde::Deserialize;

pub mod message_from_engine;

#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: AddTradePayload,
}
