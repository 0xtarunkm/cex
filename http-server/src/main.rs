use actix_web::{web, App, HttpServer};
use utils::RedisManager;

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
                .service(
                    web::scope("/order")
                        .service(routes::create_order)
                        .service(routes::delete_order)
                        .service(routes::open),
                )
                .service(web::scope("/ticker").service(routes::ticker))
                .service(web::scope("/klines").service(routes::get_klines))
                .service(web::scope("/depth").service(routes::get_depth)),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
