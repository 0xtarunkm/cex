use actix_web::{web, App, HttpServer};

mod models;
mod routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
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
