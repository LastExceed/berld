use std::io;

use protocol::packet::RegionDiscovery;

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

impl HandlePacket<RegionDiscovery> for Server {
	fn handle_packet(&self, _source: &Player, _packet: RegionDiscovery) -> Result<(), io::Error> {
		Ok(())
	}
}