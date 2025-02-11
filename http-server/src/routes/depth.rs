use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

use crate::{
    models::{GetDepthPayload, MessageToEngine, OrderType},
    state::AppState,
};

pub async fn get_depth(
    State(state): State<AppState>,
    Path((market, order_type)): Path<(String, OrderType)>,
) -> Json<Value> {
    let message = MessageToEngine::GetDepth {
        data: GetDepthPayload { market, order_type },
    };

    match state.redis_manager.send_and_wait(message) {
        Ok(response) => Json(json!(response)),
        Err(e) => Json(json!({
            "error": format!("Redis error: {}", e)
        })),
    }
}
