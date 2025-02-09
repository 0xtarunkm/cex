use actix_web::{post, web, HttpResponse, Responder};

use crate::{models::{GetUserBalancesPayload, MessageToEngine}, utils::redis_manager::RedisManager};

#[post("/balances")]
async fn get_balances(
    balances_data: web::Json<GetUserBalancesPayload>,
    redis_manager: web::Data<RedisManager>,
) -> impl Responder {
    let message = MessageToEngine::GetUserBalances {
        data: balances_data.into_inner(),
    };

    match redis_manager.send_and_wait(message) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}
