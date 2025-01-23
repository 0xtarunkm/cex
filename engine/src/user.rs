use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub balance: u64,
}

#[derive(Clone)]
pub struct UserManager {
    users: HashMap<String, User>,
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            users: HashMap::new(),
        }
    }

    pub fn create_user(&mut self, user_id: String) -> User {
        let user = User {
            user_id: user_id.clone(),
            balance: 0,
        };
        self.users.insert(user_id, user.clone());
        user
    }

    pub fn deposit(&mut self, user_id: String, amount: u64) -> Option<User> {
        if let Some(user) = self.users.get_mut(&user_id) {
            user.balance += amount;
            Some(user.clone())
        } else {
            None
        }
    }

    pub fn get_user(&self, user_id: String) -> Option<&User> {
        self.users.get(&user_id)
    }
}
