use actix_web::{delete, get, post, web, HttpResponse, Responder};

use crate::{
    models::{
        CancelOrderPayload, CreateOrderPayload, GetMarginPositionsPayload, GetOpenOrdersPayload, GetQuoteRequest, MessageToEngine
    },
    utils::redis_manager::RedisManager,
};

#[post("/create")]
async fn create_order(
    redis_manager: web::Data<RedisManager>,
    order_data: web::Json<CreateOrderPayload>,
) -> impl Responder {
    let message = MessageToEngine::CreateOrder {
        data: order_data.into_inner(),
    };

    match redis_manager.send_and_wait(message) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}

#[delete("/delete")]
async fn cancel_order(
    redis_manager: web::Data<RedisManager>,
    order_data: web::Json<CancelOrderPayload>,
) -> impl Responder {
    let message = MessageToEngine::CancelOrder {
        data: order_data.into_inner(),
    };

    match redis_manager.send_and_wait(message) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}

#[get("/get_quote")]
async fn get_quote(
    redis_manager: web::Data<RedisManager>,
    quote_data: web::Json<GetQuoteRequest>,
) -> impl Responder {
    let message = MessageToEngine::GetQuote {
        data: quote_data.into_inner(),
    };

    match redis_manager.send_and_wait(message) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}

#[get("/open")]
async fn open_orders(
    order_data: web::Json<GetOpenOrdersPayload>,
    redis_manager: web::Data<RedisManager>,
) -> impl Responder {
    let message = MessageToEngine::GetOpenOrders {
        data: order_data.into_inner(),
    };

    match redis_manager.send_and_wait(message) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}

#[get("/margin_positions")]
async fn margin_positions(
    order_data: web::Json<GetMarginPositionsPayload>,
    redis_manager: web::Data<RedisManager>
) -> impl Responder {
    let message = MessageToEngine::GetMarginPositions { data: order_data.into_inner() };

    match redis_manager.send_and_wait(message) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}
