use axum::{
    extract::{Query, State},
    Json,
};
use serde_json::{json, Value};

use crate::{
    models::{
        CancelOrderPayload, CreateOrderPayload, GetMarginPositionsPayload, GetMarginPositionsQuery,
        GetOpenOrdersPayload, GetQuoteRequest, MessageToEngine, OpenOrdersQuery,
    },
    state::AppState,
};

pub async fn create_order(
    State(state): State<AppState>,
    Json(order_data): Json<CreateOrderPayload>,
) -> Json<Value> {
    let message = MessageToEngine::CreateOrder { data: order_data };

    match state.redis_manager.send_and_wait(message) {
        Ok(response) => Json(json!(response)),
        Err(e) => Json(json!({
            "error": format!("Redis error: {}", e)
        })),
    }
}

pub async fn cancel_order(
    State(state): State<AppState>,
    Json(order_data): Json<CancelOrderPayload>,
) -> Json<Value> {
    let message = MessageToEngine::CancelOrder { data: order_data };

    match state.redis_manager.send_and_wait(message) {
        Ok(response) => Json(json!(response)),
        Err(e) => Json(json!({
            "error": format!("Redis error: {}", e)
        })),
    }
}

pub async fn get_quote(
    State(state): State<AppState>,
    Json(quote_data): Json<GetQuoteRequest>,
) -> Json<Value> {
    let message = MessageToEngine::GetQuote { data: quote_data };

    match state.redis_manager.send_and_wait(message) {
        Ok(response) => Json(json!(response)),
        Err(e) => Json(json!({
            "error": format!("Redis error: {}", e)
        })),
    }
}

pub async fn open_orders(
    State(state): State<AppState>,
    Query(params): Query<OpenOrdersQuery>,
) -> Json<Value> {
    let message = MessageToEngine::GetOpenOrders {
        data: GetOpenOrdersPayload {
            user_id: params.user_id,
            market: params.market,
        },
    };

    match state.redis_manager.send_and_wait(message) {
        Ok(response) => Json(json!(response)),
        Err(e) => Json(json!({
            "error": format!("Redis error: {}", e)
        })),
    }
}

pub async fn margin_positions(
    State(state): State<AppState>,
    Query(params): Query<GetMarginPositionsQuery>,
) -> Json<Value> {
    let message = MessageToEngine::GetMarginPositions {
        data: GetMarginPositionsPayload {
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
