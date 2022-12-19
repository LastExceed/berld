use std::io;

use protocol::packet::{Projectile, WorldUpdate};

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

impl HandlePacket<Projectile> for Server {
	fn handle_packet(&self, source: &Player, packet: Projectile) -> Result<(), io::Error> {
		self.broadcast(
			&WorldUpdate {
				projectiles: vec![packet],
				..Default::default()
			},
			Some(source)
		);

		Ok(())
	}
}