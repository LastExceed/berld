use std::io;

use protocol::packet::ZoneDiscovery;

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

impl HandlePacket<ZoneDiscovery> for Server {
	fn handle_packet(&self, _source: &Player, _packet: ZoneDiscovery) -> Result<(), io::Error> {
		Ok(())
	}
}