use axum::{
    extract::{Query, State},
    Json,
};
use serde_json::{json, Value};
use tracing::info;

use crate::{
    models::{GetUserBalancesPayload, GetUserBalancesQuery, MessageToEngine, OnRampPayload},
    state::AppState,
};

pub async fn get_balances(
    State(state): State<AppState>,
    Query(params): Query<GetUserBalancesQuery>,
) -> Json<Value> {
    let message = MessageToEngine::GetUserBalances {
        data: GetUserBalancesPayload {
            user_id: params.user_id,
        },
    };

    match state.redis_manager.send_and_wait(message) {
        Ok(response) => Json(json!(response)),
        Err(e) => Json(json!({
            "error": format!("Redis error: {}", e)
        })),
    }
}

pub async fn onramp(
    State(state): State<AppState>,
    Json(payload): Json<OnRampPayload>,
) -> Json<Value> {
    info!("hello");
    let response = state.redis_manager.onramp_and_wait(payload);

    info!("Onramp response: {:?}", response);

    match response {
        Ok(response) => Json(json!(response)),
        Err(e) => Json(json!({
            "error": format!("Redis error: {}", e)
        })),
    }
}
