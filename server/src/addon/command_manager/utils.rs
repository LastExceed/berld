use std::sync::Arc;
use crate::server::player::Player;
use crate::server::Server;

pub const INGAME_ONLY: &str = "this command can only be used ingame";

impl Server {
	#[expect(clippy::significant_drop_tightening, reason = "cannot drop any earlier")]
	pub async fn find_player(&self, query: &str) -> Option<Arc<Player>> {
		let players = self.players.read().await;

		if let Ok(id) = query.parse::<i64>()
			&& let Some(player) = players.iter().find(|player| player.id.0 == id)
		{
			return Some(Arc::clone(player));
		}

		let mut partial_match = None;
		for player in players.iter() {
			let name = player.character.read().await.name.to_lowercase();
			if name == query {
				return Some(Arc::clone(player));
			}
			if partial_match.is_none() && name.contains(query) {
				partial_match = Some(Arc::clone(player));
			}
		}

		partial_match
	}
}