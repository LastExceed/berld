use protocol::packet::{StatusEffect, WorldUpdate};
use protocol::packet::status_effect::Kind::*;
use protocol::packet::world_update::sound::Kind::*;

use crate::addon::balancing;
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<StatusEffect> for Server {
	async fn handle_packet(&self, source: &Player, mut packet: StatusEffect) {
		match packet.kind {
			Poison => {
				let Some(target) = self.find_player_by_id(packet.target).await
					else { return; };//can happen when the target disconnected in this moment

				self.apply_dot(&*(source.character.read().await), target, packet.duration / 500, 500, packet.modifier, SlimeGroan, None).await;
			}
			WarFrenzy => {
				balancing::buff_warfrenzy(&packet, self).await;
			}
			ManaShield => {
				if packet.duration == 30000 { //client echo's the packet, so we must make sure not to cause a feedback loop
					self.addons.balancing.adjust_manashield(&mut packet);
					source.send_ignoring(&WorldUpdate::from(packet.clone())).await;
				} else {
					source.notify(format!("manashield: {} ({} ms)", packet.modifier, packet.duration)).await;
				}
			}
			Affection => return, //echoed by the client when applied by the server, no need to re-broadcast
			_ => ()
		}


		self.broadcast(&WorldUpdate::from(packet), Some(source)).await;
	}
}