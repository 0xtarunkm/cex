mod models;
mod user_manager;

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use models::SocketMessage;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use user_manager::{MarketState, UserManager};

#[tokio::main]
async fn main() {
    let market_state = Arc::new(Mutex::new(UserManager::new()));
    let listener = TcpListener::bind("0.0.0.0:8080").await.unwrap();

    while let Ok((stream, addr)) = listener.accept().await {
        println!("New connection from {}", addr);
        let market_state = market_state.clone();
        tokio::spawn(handle_connection(stream, market_state));
    }
}

async fn handle_connection(stream: TcpStream, market_state: MarketState) {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    let mut user_id = String::new();

    let (tx, mut rx) = mpsc::unbounded_channel();

    tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            ws_sender
                .send(message)
                .await
                .expect("Failed to send message");
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
                            market.subscribe_to_room(user_id.clone(), room.clone(), tx.clone());
                            println!("{user_id} subscribed to {room}");
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
        println!("{user_id} disconnected");
    }
}
