use async_trait::async_trait;
use tokio::io;

use protocol::packet::ZoneDiscovery;

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<ZoneDiscovery> for Server {
	async fn handle_packet(&self, _source: &Player, _packet: ZoneDiscovery) -> Result<(), io::Error> {
		Ok(())
	}
}