use axum::{
    routing::{delete, get, post},
    Router,
};

use state::AppState;
use tracing::{error, info};

mod models;
mod routes;
mod state;
mod services;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app_state = AppState::new();

    let app = Router::new()
        .nest(
            "/api/v1",
            Router::new()
                .route("/healthcheck", get(|| async { "success: true" }))
                .nest(
                    "/order",
                    Router::new()
                        .route("/create", post(routes::create_order))
                        .route("/delete", delete(routes::cancel_order))
                        .route("/open/{user_id}/{market}", get(routes::open_orders))
                        .route("/quote", post(routes::get_quote))
                        .route("/margin_positions/{user_id}", get(routes::margin_positions)),
                )
                .nest(
                    "/user",
                    Router::new()
                        .route("/balances/{user_id}", get(routes::get_balances))
                        .route("/onramp", post(routes::onramp)),
                )
                .nest(
                    "/depth",
                    Router::new().route("/{market}/{order_type}", get(routes::get_depth)),
                )
                .nest(
                    "/ticker",
                    Router::new().route("/{market}/{order_type}", get(routes::get_ticker)),
                ),
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();

    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap_or_else(|e| {
        error!("Server error: {}", e);
        std::process::exit(1);
    });
}
