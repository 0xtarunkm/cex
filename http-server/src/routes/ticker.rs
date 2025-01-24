use actix_web::{get, web, HttpResponse, Responder};

use crate::models::GetTicker;

#[get("/")]
async fn ticker(data: web::Json<GetTicker>) -> impl Responder {
    HttpResponse::Ok().json("ticker")
}
