use config::{Config, ConfigError};

use crate::server::player::Player;
use crate::server::Server;

pub mod legacy_koth;
pub mod shop;
pub mod utils;

pub struct Events {
    pub legacy_koth: legacy_koth::LegacyKoth,
    pub shop: shop::ItemShop
}

impl Events {
    pub fn new(config: &Config) -> Result<Self, ConfigError> {
        Ok(Self {
            legacy_koth: legacy_koth::LegacyKoth::new(config)?,
            shop: shop::ItemShop::new(config)?
        })
    }
}

pub fn start() {
    legacy_koth::start()
}

pub async fn on_kill(server: &Server, killer: &Player, victim: &Player) {
    legacy_koth::on_kill(server, killer, victim).await
}

pub async fn on_join(player: &Player) {
    shop::on_join(player).await;
    legacy_koth::on_join(player).await;
}