use std::{collections::HashMap, sync::Arc};

use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use serde_json;

use crate::models::{MessageTx, SocketMessage};

pub struct Market {
    clients: HashMap<String, MessageTx>,
}

impl Market {
    pub fn new() -> Self {
        Market {
            clients: HashMap::new(),
        }
    }

    pub fn add_client(&mut self, user_id: String, tx: MessageTx) -> bool {
        if self.clients.contains_key(&user_id) {
            return false;
        }
        self.clients.insert(user_id, tx);
        true
    }

    pub fn remove_client(&mut self, user_id: &str) {
        self.clients.remove(user_id);
    }

    pub fn broadcast(&self, sender: &str, message: &str) {
        for (username, tx) in self.clients.iter() {
            if username != sender {
                let _ = tx.send(Message::Text(message.to_string().into()));
            }
        }
    }
}

pub struct UserManager {
    rooms: HashMap<String, Market>,
    user_rooms: HashMap<String, Vec<String>>,
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            rooms: HashMap::new(),
            user_rooms: HashMap::new(),
        }
    }

    pub fn subscribe_to_room(&mut self, user_id: String, room: String, tx: MessageTx) {
        if !self.rooms.contains_key(&room) {
            self.rooms.insert(room.clone(), Market::new());
        }

        if let Some(room) = self.rooms.get_mut(&room) {
            room.add_client(user_id.clone(), tx);
        }

        self.user_rooms
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(room);
    }

    pub fn send_message(&self, sender: &str, room: &str, message: &str) -> bool {
        if let Some(room) = self.rooms.get(room) {
            room.broadcast(sender, message);
            true
        } else {
            false
        }
    }

    pub fn remove_user(&mut self, user_id: &str) {
        if let Some(user_rooms) = self.user_rooms.get(user_id) {
            for room_name in user_rooms {
                if let Some(room) = self.rooms.get_mut(room_name) {
                    room.remove_client(user_id);
                }
            }
        }
        self.user_rooms.remove(user_id);
    }

    pub fn broadcast_redis_message(&mut self, channel: &str, payload: &str) {
        let message = serde_json::to_string(&SocketMessage::SendMessage {
            message: payload.to_string(),
            room: channel.to_string(),
        }).unwrap();
        
        if let Some(senders) = self.rooms.get_mut(channel) {
            senders.clients.retain(|_, tx| {
                tx.send(Message::Text(message.clone()))
                    .is_ok()
            });
        }
    }

    pub fn broadcast_to_room(&mut self, room: &str, message: &str) {
        println!("Broadcasting WebSocket message to room: {}", room);
        let room = room.trim();
        if let Some(subscribers) = self.rooms.get(room) {
            println!("Found room with {} clients", subscribers.clients.len());
            let formatted_message = serde_json::json!({
                "type": "WS_MESSAGE",
                "room": room,
                "message": message
            }).to_string();
            
            for (client_id, tx) in subscribers.clients.iter() {
                println!("Sending WebSocket message to client {}", client_id);
                if let Err(e) = tx.send(Message::Text(formatted_message.clone())) {
                    println!("Failed to send to client {}: {}", client_id, e);
                }
            }
        }
    }

    pub fn has_subscription(&self, user_id: &str, room: &str) -> bool {
        self.rooms
            .get(room)
            .map_or(false, |market| market.clients.iter().any(|(uid, _)| uid == user_id))
    }
}

pub type MarketState = Arc<Mutex<UserManager>>;
