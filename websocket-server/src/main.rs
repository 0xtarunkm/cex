mod models;
mod redis_manager;
mod user_manager;

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use models::SocketMessage;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use tracing::info;
use user_manager::{MarketState, UserManager};

#[allow(deprecated)]
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let market_state = Arc::new(Mutex::new(UserManager::new()));
    let listener = TcpListener::bind("0.0.0.0:8081").await.unwrap();

    let market_state_redis = market_state.clone();
    tokio::spawn(async move {
        let redis = redis_manager::RedisManager::instance();
        let conn = redis.get_async_connection().await.unwrap();
        let mut pubsub = conn.into_pubsub();

        pubsub.psubscribe("trade@*").await.unwrap();
        pubsub.psubscribe("depth@*").await.unwrap();
        info!("Redis subscription started");

        loop {
            if let Some(msg) = pubsub.on_message().next().await {
                let channel = msg.get_channel::<String>().unwrap();
                if let Ok(payload) = msg.get_payload::<String>() {
                    info!("Redis received on {}: {}", channel, payload);
                    let mut market = market_state_redis.lock().await;
                    market.broadcast_redis_message(&channel, &payload);
                }
            }
        }
    });

    while let Ok((stream, addr)) = listener.accept().await {
        info!("New connection from {}", addr);
        let market_state = market_state.clone();
        tokio::spawn(handle_connection(stream, market_state));
    }
}

async fn handle_connection(stream: TcpStream, market_state: MarketState) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let mut user_id = String::new();

    let (tx, mut rx) = mpsc::unbounded_channel();

    let ping_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            if let Err(e) = ping_tx.send(Message::Ping(vec![])) {
                info!("Failed to send ping: {:?}", e);
                break;
            }
        }
    });

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = ws_sender.send(message).await {
                info!("Failed to send WebSocket message: {:?}", e);
                break;
            }
        }
    });

    while let Some(msg) = ws_receiver.next().await {
        if let Ok(msg) = msg {
            if let Message::Text(text) = msg {
                if let Ok(socket_message) = serde_json::from_str::<SocketMessage>(&text) {
                    let mut market = market_state.lock().await;
                    match socket_message {
                        SocketMessage::Subscribe { room } => {
                            if user_id.is_empty() {
                                user_id = format!("user_{}", rand::random::<u32>());
                            }
                            if !market.has_subscription(&user_id, &room) {
                                market.subscribe_to_room(user_id.clone(), room.clone(), tx.clone());
                                info!("{user_id} subscribed to {room}");
                            }
                        }
                        SocketMessage::SendMessage { message, room } => {
                            if !user_id.is_empty() {
                                market.send_message(&user_id, &room, &message);
                            }
                        }
                    }
                }
            }
        }
    }

    if !user_id.is_empty() {
        let mut market = market_state.lock().await;
        market.remove_user(&user_id);
        info!("{user_id} disconnected");
    }
}
