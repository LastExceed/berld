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

		let players_guard = self.players.read().await;
		let Some(target) = players_guard.iter().find(|player| { player.id == packet.target }) else {
			return Ok(()) //can happen when the target disconnected in this moment
		};

		target.send_ignoring(&WorldUpdate::from(packet)).await; //todo: only target needs to receive this packet, but finding player by id is expensive atm

		Ok(())
	}
}