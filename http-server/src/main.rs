use axum::{
    routing::{delete, get, post},
    Router,
};

use state::AppState;
use tracing::{error, info};

mod models;
mod routes;
mod services;
mod state;

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
                        .route("/cancel", delete(routes::cancel_order))
                        .route("/open", get(routes::open_orders))
                        .route("/quote", post(routes::get_quote))
                        .route("/margin-positions", get(routes::margin_positions)),
                )
                .nest(
                    "/user",
                    Router::new()
                        .route("/balances", get(routes::get_balances))
                        .route("/onramp", post(routes::onramp)),
                )
                .route("/depth", get(routes::get_depth))
                .route("/ticker", get(routes::get_ticker)),
        )
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap_or_else(|e| {
        error!("Server error: {}", e);
        std::process::exit(1);
    });
}
