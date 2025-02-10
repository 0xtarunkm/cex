use std::{sync::Arc, time::Duration};

use log::{error, info};
use rust_decimal::Decimal;
use tokio::{sync::Mutex, time};

use crate::models::{MarginPosition, OrderType, User};

use super::price_service::PriceService;

pub struct PnlMonitor {
    users: Arc<Mutex<Vec<User>>>,
    price_service: Arc<PriceService>,
}

impl PnlMonitor {
    pub fn new(users: Arc<Mutex<Vec<User>>>, price_service: Arc<PriceService>) -> Self {
        PnlMonitor {
            users,
            price_service,
        }
    }

    pub async fn start_monitoring(&self) {
        let mut interval = time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            if let Err(e) = self.update_all_positions().await {
                error!("Error updating positions: {}", e);
            }
        }
    }

    async fn update_all_positions(&self) -> Result<(), String> {
        let mut users = self.users.lock().await;

        for user in users.iter_mut() {
            let mut liquidate = false;
            let mut position_to_liquidate = None;
            let mut liquidation_price = None;

            for position in user.margin_positions.iter_mut() {
                let market = format!("{}_USDC", position.asset);
                if let Some(price) = self.price_service.get_price(&market).await {
                    position.calculate_unrealized_pnl(price);

                    if let Some(liq_price) = position.liquidation_price {
                        match position.position_type {
                            OrderType::MarginLong if price <= liq_price => {
                                liquidate = true;
                                position_to_liquidate = Some(position.clone());
                                liquidation_price = Some(price);
                                break;
                            }
                            OrderType::MarginShort if price >= liq_price => {
                                liquidate = true;
                                position_to_liquidate = Some(position.clone());
                                liquidation_price = Some(price);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }

            if liquidate {
                if let (Some(pos), Some(price)) = (position_to_liquidate, liquidation_price) {
                    self.liquidate_position(user, &pos, price).await?;
                }
            }
        }
        Ok(())
    }

    async fn liquidate_position(
        &self,
        user: &mut User,
        position: &MarginPosition,
        price: Decimal,
    ) -> Result<(), String> {
        info!(
            "Liquidating position for user {}: {} {} @ {}",
            user.id, position.quantity, position.asset, price
        );

        let realized_pnl = match position.position_type {
            OrderType::MarginLong => (price - position.avg_price) * position.quantity,
            OrderType::MarginShort => (position.avg_price - price) * position.quantity,
            _ => return Err("Invalid position type".to_string()),
        };

        user.realized_pnl += realized_pnl;
        user.margin_positions
            .retain(|p| p.asset != position.asset || p.position_type != position.position_type);

        Ok(())
    }
}
