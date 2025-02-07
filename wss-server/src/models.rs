use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    Join { username: String },
    Send { username: String, content: String },
    Leave { username: String },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SocketMessage {
    #[serde(rename = "SUBSCRIBE")]
    Subscribe { room: String },
    #[serde(rename = "SEND_MESSAGE")]
    SendMessage { message: String, room: String },
}

pub type MessageTx = mpsc::UnboundedSender<Message>;
