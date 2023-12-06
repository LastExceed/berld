use protocol::packet::{WorldUpdate, ZoneDiscovery};
use protocol::packet::world_update::p48::P48sub;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<ZoneDiscovery> for Server {
	async fn handle_packet(&self, source: &Player, packet: ZoneDiscovery) {
		let p48sub = P48sub([0_u8; 16]);
		let world_update = WorldUpdate {
			//todo: filter to just this + adjacent zones
			loot: self.loot.read().await.clone(),
			//in case there are no items in the current zone
			//we still want the client to stop asking
			..(packet.0, vec![p48sub]).into()
		};

		source.send_ignoring(&world_update).await;
	}
}