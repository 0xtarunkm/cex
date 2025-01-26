mod messages;
mod state;
mod subscriptions;
mod user;

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    routing::get,
    Router,
};
use std::{collections::HashMap, sync::Arc};
use subscriptions::SubscriptionManager;
use tokio::{net::TcpListener, sync::RwLock};

use crate::messages::{IncomingMessage, IncomingMethod};
use crate::state::AppState;
use crate::user::User;

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> axum::response::Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let user = User::new();
    let user_id = user.id;

    {
        let mut users = state.users.write().await;
        users.insert(user_id, user.clone());
    }

    while let Some(Ok(msg)) = socket.recv().await {
        if let axum::extract::ws::Message::Text(text) = msg {
            if let Ok(incoming_msg) = serde_json::from_str::<IncomingMessage>(&text) {
                match incoming_msg.method {
                    IncomingMethod::Subscribe => {
                        for topic in incoming_msg.params {
                            state.subscription_manager.subscribe(user_id, topic).await;
                        }
                    }
                    IncomingMethod::Unsubscribe => {
                        for topic in incoming_msg.params {
                            state
                                .subscription_manager
                                .unsubscribe(user_id, &topic)
                                .await;
                        }
                    }
                }
            }
        }
    }

    {
        let mut users = state.users.write().await;
        users.remove(&user_id);
    }
}

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let subscription_manager = Arc::new(SubscriptionManager::new().await);

    let app_state = AppState {
        users: Arc::new(RwLock::new(HashMap::new())),
        subscription_manager: subscription_manager.clone(),
    };

    let app = Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(app_state);

    println!("Server listening on port 3001");
    let listener = TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
