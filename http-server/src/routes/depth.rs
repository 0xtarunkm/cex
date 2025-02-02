use actix_web::{post, web, HttpResponse, Responder};

use crate::{
    models::{GetDepthData, MessageToEngine}, utils::redis_manager::RedisManager
};

#[post("/")]
async fn get_depth(
    depth_data: web::Json<GetDepthData>,
    redis_manager: web::Data<RedisManager>,
) -> impl Responder {
    let message = MessageToEngine::GetDepth {
        data: depth_data.into_inner(),
    }; 

    match redis_manager.send_and_wait(message) {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}
