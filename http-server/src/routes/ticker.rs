use actix_web::{get, HttpResponse, Responder};

#[get("/all")]
async fn get_tickers() -> impl Responder {
    HttpResponse::Ok().json("success: all tickers")
}
