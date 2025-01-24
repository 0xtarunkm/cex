use actix_web::{get, web, HttpResponse, Responder};

use crate::{
    lib::RedisManager,
    models::{GetDepthData, MessageToEngine},
};

#[get("/")]
pub async fn get_depth(
    redis_manager: web::Data<RedisManager>,
    depth_data: web::Json<GetDepthData>,
) -> impl Responder {
    let message = MessageToEngine::GetDepth {
        data: depth_data.into_inner(),
    };

    match redis_manager.send_and_await(message).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => HttpResponse::InternalServerError().body(format!("Redis error: {}", e)),
    }
}
