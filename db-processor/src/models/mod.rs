use message_from_engine::MessageFromEngine;
use serde::Deserialize;

pub mod message_from_engine;

#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    pub message: MessageFromEngine,
}
