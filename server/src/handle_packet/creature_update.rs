use async_trait::async_trait;

use protocol::packet::CreatureUpdate;

use crate::addons::anti_cheat;
use crate::addons::enable_pvp;
use crate::addons::traffic_filter::filter;
use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<CreatureUpdate> for Server {
	async fn handle_packet(&self, source: &Player, mut packet: CreatureUpdate) {
		let mut character = source.creature.write().await;
		let snapshot = character.clone();
		character.update(&packet);
		drop(character);
		let character = source.creature.read().await;//todo: downgrade character lock

		if let Err(message) = anti_cheat::inspect_creature_update(&packet, &snapshot, &character) {
			dbg!(&message);
			self.kick(source, message).await;
			return;
		}

		enable_pvp(&mut packet);

		if filter(&mut packet, &snapshot, &character) {
			//todo: move somewhere else
			if let Some(animation_time) = packet.animation_time && animation_time <= snapshot.animation_time {
				packet.animation_time = Some(0); //starts all animations from the beginning to prevent cut-off animations, at the cost of some minimal delay
			}

			self.broadcast(&packet, Some(source)).await;
		}
	}
}