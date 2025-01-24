use actix_web::{delete, get, post, web, HttpResponse, Responder};

use crate::{
    lib::RedisManager,
    models::{CancelOrderData, CreateOrderData, GetOpenOrdersData, MessageToEngine},
};

#[post("/create")]
pub async fn create_order(
    redis_manager: web::Data<RedisManager>,
    order_data: web::Json<CreateOrderData>,
) -> impl Responder {
    let message = MessageToEngine::CreateOrder {
        data: order_data.into_inner(),
    };

    match redis_manager.send_and_await(message).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}

#[delete("/delete")]
pub async fn delete_order(
    redis_manager: web::Data<RedisManager>,
    order_data: web::Json<CancelOrderData>,
) -> impl Responder {
    let message = MessageToEngine::CancelOrder {
        data: order_data.into_inner(),
    };

    match redis_manager.send_and_await(message).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}

#[get("/open")]
pub async fn open(
    redis_manager: web::Data<RedisManager>,
    order_data: web::Json<GetOpenOrdersData>,
) -> impl Responder {
    let message = MessageToEngine::GetOpenOrders {
        data: order_data.into_inner(),
    };

    match redis_manager.send_and_await(message).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}
