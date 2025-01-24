use actix_web::{delete, get, post, web, HttpResponse, Responder};

use crate::models::{CreateOrder, DeleteOrder};

#[post("/create")]
pub async fn create_order(order: web::Json<CreateOrder>) -> impl Responder {
    HttpResponse::Ok().json("Order added")
}

#[delete("/delete")]
pub async fn delete_order(order: web::Json<DeleteOrder>) -> impl Responder {
    HttpResponse::Ok().json("Order deleted")
}

#[get("/open")]
pub async fn open() -> impl Responder {
    HttpResponse::Ok().json("open orders")
}
