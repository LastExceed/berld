use std::io;

use protocol::packet::{StatusEffect, WorldUpdate};

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

impl HandlePacket<StatusEffect> for Server {
	fn handle_packet(&self, source: &Player, packet: StatusEffect) -> Result<(), io::Error> {
		self.broadcast(
			&WorldUpdate {
				status_effects: vec![packet],
				..Default::default()
			},
			Some(source)
		);

		Ok(())
	}
}