use std::sync::Arc;
use crate::server::player::Player;
use crate::server::Server;

pub const INGAME_ONLY: &str = "this command can only be used ingame";

impl Server {
	pub async fn find_player(&self, query: &str) -> Option<Arc<Player>> {
		let players = self.players.read().await;

		if let Ok(id) = query.parse::<i64>() {
			if let Some(player) = players.iter().find(|player| player.id.0 == id) {
				return Some(player.clone());
			}
		}

		for player in players.iter() {
			let matches_query = player.character
				.read().await
				.name
				.to_lowercase()
				.contains(query);
			if matches_query {
				return Some(player.clone())
			}
		}

		None
	}
}