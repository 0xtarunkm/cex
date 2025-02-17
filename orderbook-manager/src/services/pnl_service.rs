use std::{sync::Arc, time::Duration};

use rust_decimal::Decimal;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::models::{MarginPosition, PositionType, User};

use super::price_service::PriceService;

pub struct PnlService {
    users: Arc<RwLock<Vec<User>>>,
    price_service: Arc<PriceService>,
}

impl PnlService {
    pub fn new(users: Arc<RwLock<Vec<User>>>, price_service: Arc<PriceService>) -> Self {
        PnlService {
            users,
            price_service,
        }
    }

    pub fn start_monitoring(&self) {
        let users = self.users.clone();
        let price_service = self.price_service.clone();

        std::thread::Builder::new()
            .name("pnl_monitor".to_string())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(1));
                    info!("Started PNL monitoring thread");

                    loop {
                        interval.tick().await;
                        if let Err(e) = Self::check_positions(&users, &price_service).await {
                            error!("Error checking positions: {}", e);
                        }
                    }
                });
            })
            .expect("Failed to spawn PNL monitoring thread");
    }

    async fn check_positions(
        users: &Arc<RwLock<Vec<User>>>,
        price_service: &Arc<PriceService>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut users = users.write().await;

        for user in users.iter_mut() {
            let mut positions_to_liquidate = Vec::new();

            for position in user.margin_positions.iter() {
                let market = format!("{}_USDC", position.asset);
                if let Some(price) = price_service.get_price(&market).await {
                    let liquidation_threshold = position.collateral * Decimal::new(-80, 2);
                    if position.unrealized_pnl <= liquidation_threshold {
                        positions_to_liquidate.push((position.clone(), price));
                    }
                }
            }

            for (position, price) in positions_to_liquidate {
                Self::liquidate_position(user, &position, price).await?;
            }
        }
        Ok(())
    }

    async fn liquidate_position(
        user: &mut User,
        position: &MarginPosition,
        price: Decimal,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            user_id = ?user.id,
            asset = ?position.asset,
            size = ?position.size,
            price = ?price,
            "Liquidating position"
        );

        let realized_pnl = match position.position_type {
            PositionType::Long => (price - position.entry_price) * position.size,
            PositionType::Short => (position.entry_price - price) * position.size,
        };

        user.realized_pnl += realized_pnl;

        if let Some(usdc_balance) = user.balances.iter_mut().find(|b| b.ticker == "USDC") {
            usdc_balance.balance += realized_pnl + position.collateral;
            usdc_balance.locked_balance -= position.collateral;
        }

        if position.position_type == PositionType::Short {
            if let Some(asset_balance) = user
                .balances
                .iter_mut()
                .find(|b| b.ticker == position.asset)
            {
                asset_balance.locked_balance -= position.size;
            }
        }

        user.margin_used -= position.collateral;

        user.margin_positions
            .retain(|p| p.asset != position.asset || p.position_type != position.position_type);

        Ok(())
    }
}
