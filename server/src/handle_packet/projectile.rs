use async_trait::async_trait;
use tokio::io;

use protocol::packet::{Projectile, WorldUpdate};

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<Projectile> for Server {
	async fn handle_packet(&self, source: &Player, packet: Projectile) -> io::Result<()> {
		self.broadcast(
			&WorldUpdate {
				projectiles: vec![packet],
				..Default::default()
			},
			Some(source)
		).await;

		Ok(())
	}
}