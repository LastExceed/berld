use std::io;
use std::io::ErrorKind;

use async_trait::async_trait;

use protocol::packet::CreatureUpdate;

use crate::anti_cheat;
use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::pvp::enable_pvp;
use crate::server::Server;
use crate::traffic_filter::filter;

#[async_trait]
impl HandlePacket<CreatureUpdate> for Server {
	async fn handle_packet(&self, source: &Player, mut packet: CreatureUpdate) -> Result<(), io::Error> {
		enable_pvp(&mut packet);

		let mut character = source.creature.write().await;
		let snapshot = character.clone();
		character.update(&packet);
		//todo: downgrade character lock

		if let Err(message) = anti_cheat::inspect_creature_update(&packet, &snapshot, &character) {
			dbg!(&message);
			self.kick(&source, message).await;
			return Err(ErrorKind::InvalidInput.into())
		}

		if filter(&mut packet, &snapshot, &character) {
			self.broadcast(&packet, Some(source)).await;
		}

		Ok(())
	}
}