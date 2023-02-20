use protocol::packet::{Projectile, WorldUpdate};

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<Projectile> for Server {
	async fn handle_packet(&self, source: &Player, packet: Projectile) {
		self.broadcast(&WorldUpdate::from(packet), Some(source)).await;
	}
}