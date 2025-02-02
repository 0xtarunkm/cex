use actix_web::{delete, get, post, web, HttpResponse, Responder};

use crate::{
    models::{CancelOrderData, CreateOrderData, GetOpenOrdersData, MessageToEngine}, utils::redis_manager::RedisManager
};

#[post("/create")]
async fn create_order(
    redis_manager: web::Data<RedisManager>,
    order_data: web::Json<CreateOrderData>,
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
async fn delete_order(
    redis_manager: web::Data<RedisManager>,
    order_data: web::Json<CancelOrderData>,
) -> impl Responder {
    let message = MessageToEngine::CancelOrder {
        data: order_data.into_inner(),
    };

    match redis_manager.send_and_wait(message) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}

#[get("/open")]
async fn open_orders(
    order_data: web::Json<GetOpenOrdersData>,
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
