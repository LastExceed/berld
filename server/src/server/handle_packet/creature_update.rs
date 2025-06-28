use std::sync::atomic::Ordering;

use protocol::packet::CreatureUpdate;

use crate::addon::{anti_cheat, kill_feed, pvp};
use crate::addon::fix_cutoff_animations;
use crate::addon::traffic_filter::filter;
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<CreatureUpdate> for Server {
	#[expect(clippy::significant_drop_tightening, reason = "false positive")]
	async fn handle_packet(&self, source: &Player, mut packet: CreatureUpdate) {
		if let Err(message) = anti_cheat::inspect_creature_update(source, &packet).await && !source.ac_immune.load(Ordering::Relaxed) {
			self.kick(source, message).await;
			return;
		}

		self.addons.balancing.track_airtime(source).await;
		pvp::on_creature_update(self, source, &packet).await;
		kill_feed::on_creature_update(self, source, &packet).await;

		let mut character = source.character.write().await;
		let snapshot = character.clone();
		character.update(&packet);
		let character = character.downgrade();

		if !filter(&mut packet, &snapshot, &character) {
			return;
		}
		drop(character);

		fix_cutoff_animations(&mut packet, &snapshot);

		if pvp::broadcast(self, source, &packet).await {
			return;
		}

		self.broadcast(&packet, Some(source)).await;
	}
}