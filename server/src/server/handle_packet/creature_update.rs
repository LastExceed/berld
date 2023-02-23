use protocol::packet::CreatureUpdate;

use crate::addons::{anti_cheat, fix_cutoff_animations};
use crate::addons::enable_pvp;
use crate::addons::traffic_filter::filter;
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<CreatureUpdate> for Server {
	async fn handle_packet(&self, source: &Player, mut packet: CreatureUpdate) {
		let mut character = source.creature.write().await;
		let snapshot = character.clone();
		character.update(&packet);
		let character = character.downgrade();

		if let Err(message) = anti_cheat::inspect_creature_update(&packet, &snapshot, &character) {
			dbg!(&message);
			self.kick(source, message).await;
			return;
		}

		enable_pvp(&mut packet);

		if !filter(&mut packet, &snapshot, &character) {
			return;
		}

		fix_cutoff_animations(&mut packet, &snapshot);
		self.broadcast(&packet, Some(source)).await;
	}
}