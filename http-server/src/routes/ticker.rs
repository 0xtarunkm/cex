use actix_web::{get, HttpResponse, Responder};

#[get("/")]
async fn ticker() -> impl Responder {
    // query it from the timescaledb trades table
    HttpResponse::Ok().json("ticker")
}
