use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GetTicker {
    market: String,
}
