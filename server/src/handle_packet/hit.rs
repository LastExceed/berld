use async_trait::async_trait;
use tokio::io;

use protocol::packet::{Hit, WorldUpdate};

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<Hit> for Server {
	async fn handle_packet(&self, source: &Player, packet: Hit) -> io::Result<()> {
		if packet.target == packet.attacker && packet.damage.is_sign_negative() {
			return Ok(()) //self-heal is already applied client-side (which is a bug) so we need to ignore it server-side
		}

		self.broadcast(&WorldUpdate::from(packet), Some(source)).await; //todo: only target needs to receive this packet, but finding player by id is expensive atm

		Ok(())
	}
}