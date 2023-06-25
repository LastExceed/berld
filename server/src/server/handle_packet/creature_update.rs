use protocol::packet::CreatureUpdate;

use crate::addon::enable_pvp;
use crate::addon::fix_cutoff_animations;
use crate::addon::traffic_filter::filter;
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<CreatureUpdate> for Server {
	async fn handle_packet(&self, source: &Player, mut packet: CreatureUpdate) {
		let mut character = source.character.write().await;
		let snapshot = character.clone();
		character.update(&packet);
		let character = character.downgrade();

		if let Err(message) = self.addons.anti_cheat.inspect_creature_update(source, &packet, &snapshot, &character).await {
			self.kick(source, message).await;
			return;
		}

		if !filter(&mut packet, &snapshot, &character) {
			return;
		}

		enable_pvp(&mut packet);
		fix_cutoff_animations(&mut packet, &snapshot);
		self.addons.air_time_tracker.on_creature_update(source).await;

		self.broadcast(&packet, Some(source)).await;
	}
}