use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct IncomingMessage {
    pub client_id: String,
    pub user_id: i32,
    pub message: WalletMessage,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct WalletMessage {
    pub network: String,
    pub token: String,
    pub user_id: String,
}
