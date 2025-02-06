use actix_web::{post, web, HttpResponse, Responder};

use crate::{
    models::{MessageToEngine, OnRampData},
    utils::redis_manager::RedisManager,
};

#[post("/onramp")]
async fn onramp(
    onramp_data: web::Json<OnRampData>,
    redis_manager: web::Data<RedisManager>,
) -> impl Responder {
    let message = MessageToEngine::OnRamp {
        data: onramp_data.into_inner(),
    };

    match redis_manager.send_and_wait(message) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}
