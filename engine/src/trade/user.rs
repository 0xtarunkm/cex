use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBalance {
    pub available: HashMap<String, f64>, // Available balances per asset
    pub reserved: HashMap<String, f64>,  // Reserved balances per asset
}

impl UserBalance {
    pub fn new() -> Self {
        UserBalance {
            available: HashMap::new(),
            reserved: HashMap::new(),
        }
    }

    pub fn deposit(&mut self, asset: &str, amount: f64) {
        *self.available.entry(asset.to_string()).or_insert(0.0) += amount;
    }

    pub fn reserve(&mut self, asset: &str, amount: f64) -> bool {
        let available_balance = self.available.entry(asset.to_string()).or_insert(0.0);
        if *available_balance >= amount {
            *available_balance -= amount;
            *self.reserved.entry(asset.to_string()).or_insert(0.0) += amount;
            true
        } else {
            false
        }
    }

    pub fn release(&mut self, asset: &str, amount: f64) {
        let reserved_balance = self.reserved.entry(asset.to_string()).or_insert(0.0);
        if *reserved_balance >= amount {
            *reserved_balance -= amount;
            *self.available.entry(asset.to_string()).or_insert(0.0) += amount;
        }
    }

    pub fn settle(&mut self, asset: &str, amount: f64) {
        let reserved_balance = self.reserved.entry(asset.to_string()).or_insert(0.0);
        if *reserved_balance >= amount {
            *reserved_balance -= amount;
        }
    }
}
