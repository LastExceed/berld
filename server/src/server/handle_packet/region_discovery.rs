use protocol::packet::RegionDiscovery;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<RegionDiscovery> for Server {
	#[expect(clippy::unused_async, reason="False positive rust-lang/rust-clippy#10459")]
	async fn handle_packet(&self, _source: &Player, _packet: RegionDiscovery) {

	}
}