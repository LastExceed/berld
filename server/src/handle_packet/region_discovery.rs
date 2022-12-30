use async_trait::async_trait;
use tokio::io;

use protocol::packet::RegionDiscovery;

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<RegionDiscovery> for Server {
	async fn handle_packet(&self, _source: &Player, _packet: RegionDiscovery) -> Result<(), io::Error> {
		Ok(())
	}
}