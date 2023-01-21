use async_trait::async_trait;

use protocol::packet::{Projectile, WorldUpdate};

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<Projectile> for Server {
	async fn handle_packet(&self, source: &Player, packet: Projectile) {
		self.broadcast(&WorldUpdate::from(packet), Some(source)).await;
	}
}