use protocol::packet::{AreaRequest, WorldUpdate};
use protocol::packet::area_request::Zone;
use protocol::packet::world_update::p48::P48sub;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<AreaRequest<Zone>> for Server {
	async fn handle_packet(&self, source: &Player, packet: AreaRequest<Zone>) {
		let p48sub = P48sub([0_u8; 16]);
		let world_update = WorldUpdate {
			//todo: filter to just this + adjacent zones
			loot: self.loot.read().await.clone(),
			blocks: self.addons.models.blocks_in(packet.0),
			p48: [(packet.0, vec![p48sub])].into(),
			..Default::default()
		};

		source.send_ignoring(&world_update).await;
	}
}