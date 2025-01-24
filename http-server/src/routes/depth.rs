use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GetDepth {
    symbol: String,
}

#[get("/")]
pub async fn get_depth(data: web::Json<GetDepth>) -> impl Responder {
    HttpResponse::Ok().json("depth")
}
