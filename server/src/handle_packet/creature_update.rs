use std::io;
use std::io::ErrorKind;

use parking_lot::lock_api::RawRwLockDowngrade;

use protocol::packet::CreatureUpdate;

use crate::anti_cheat;
use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::pvp::enable_pvp;
use crate::server::Server;
use crate::traffic_filter::filter;

impl HandlePacket<CreatureUpdate> for Server {
	fn handle_packet(&self, source: &Player, mut packet: CreatureUpdate) -> Result<(), io::Error> {
		enable_pvp(&mut packet);

		let mut character = source.creature.write();
		let snapshot = character.clone();
		character.update(&packet);
		unsafe { source.creature.raw().downgrade(); }//todo: not sure

		if let Err(message) = anti_cheat::inspect_creature_update(&packet, &snapshot, &character) {
			dbg!(&message);
			self.kick(&source, message);
			return Err(ErrorKind::InvalidInput.into())
		}

		if filter(&mut packet, &snapshot, &character) {
			self.broadcast(&packet, Some(source));
		}

		Ok(())
	}
}