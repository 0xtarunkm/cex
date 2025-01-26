use actix_web::{get, web, HttpResponse, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct GetKlines {
    market: String,
    interval: u64,
    start_time: u64,
    end_time: u64,
}

#[get("/")]
pub async fn get_klines(kline: web::Json<GetKlines>) -> impl Responder {
    // query the timescale db for trades
    HttpResponse::Ok().json("klines")
}
