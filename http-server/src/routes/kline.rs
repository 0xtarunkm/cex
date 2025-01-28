use actix_web::{get, HttpResponse, Responder};

#[get("/get")]
async fn get_klines() -> impl Responder {
    HttpResponse::Ok().json("success: klines data")
}
