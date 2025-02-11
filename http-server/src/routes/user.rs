use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

use crate::{
    models::{GetUserBalancesPayload, MessageToEngine},
    state::AppState,
};

pub async fn get_balances(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Json<Value> {
    let message = MessageToEngine::GetUserBalances {
        data: GetUserBalancesPayload { user_id },
    };

    match state.redis_manager.send_and_wait(message) {
        Ok(response) => Json(json!(response)),
        Err(e) => Json(json!({
            "error": format!("Redis error: {}", e)
        })),
    }
}
