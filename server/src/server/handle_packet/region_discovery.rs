use protocol::packet::RegionDiscovery;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<RegionDiscovery> for Server {
	async fn handle_packet(&self, _source: &Player, _packet: RegionDiscovery) {

	}
}