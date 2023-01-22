use async_trait::async_trait;

use protocol::packet::ZoneDiscovery;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<ZoneDiscovery> for Server {
	async fn handle_packet(&self, _source: &Player, _packet: ZoneDiscovery) {

	}
}