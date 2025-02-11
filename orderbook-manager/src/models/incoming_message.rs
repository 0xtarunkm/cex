use super::MessageFromApi;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    pub client_id: String,
    pub message: MessageFromApi,
}
