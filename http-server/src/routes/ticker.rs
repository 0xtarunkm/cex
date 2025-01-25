use actix_web::{get, HttpResponse, Responder};

#[get("/")]
async fn ticker() -> impl Responder {
    HttpResponse::Ok().json("ticker")
}
