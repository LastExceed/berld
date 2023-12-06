use protocol::packet::AreaRequest;
use protocol::packet::area_request::Region;
use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<AreaRequest<Region>> for Server {
	async fn handle_packet(&self, _source: &Player, _packet: AreaRequest<Region>) {

	}
}