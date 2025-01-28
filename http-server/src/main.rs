use actix_web::{web, App, HttpResponse, HttpServer};
use utils::redis_manager::RedisManager;

mod models;
mod routes;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let redis_manager = web::Data::new(RedisManager::new());

    HttpServer::new(move || {
        App::new().app_data(redis_manager.clone()).service(
            web::scope("/api/v1")
                .route(
                    "/healthcheck",
                    web::get().to(|| async { HttpResponse::Ok().json("success: true") }),
                )
                .service(
                    web::scope("/order")
                        .service(routes::create_order)
                        .service(routes::delete_order)
                        .service(routes::open_orders),
                )
                .service(web::scope("/depth").service(routes::get_depth))
                .service(web::scope("/klines").service(routes::get_klines))
                .service(web::scope("/tickers").service(routes::get_tickers))
                .service(web::scope("/user").service(routes::onramp)),
        )
    })
    .workers(4)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
