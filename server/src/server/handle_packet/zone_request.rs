use std::time::{Duration, Instant};

use tokio::time::sleep;

use protocol::packet::{AreaRequest, WorldUpdate};
use protocol::packet::area_request::Zone;
use protocol::utils::constants::SIZE_ZONE;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;
use crate::SERVER;

// a client's zone request is not proof of a zone being loaded
// a client's silence after receiving a safe zone update is
const CONFIRMATION_WINDOW: Duration = Duration::from_millis(1500);
// max radius a zone can remain "loaded" client-side
const RETENTION_RADIUS: i32 = 3;

impl HandlePacket<AreaRequest<Zone>> for Server {
	async fn handle_packet(&self, source: &Player, packet: AreaRequest<Zone>) {
		let zone = packet.0;

		let zone_loot = self.loot.read().await.get(&zone).cloned().unwrap_or_default();
		source.send_ignoring(&WorldUpdate::from((zone, zone_loot))).await;

		// block updates can crash the client if sent too early
		if !self.addons.models.has_model(zone) {
			return;
		}

		let requested_at = Instant::now();
		source.addon_data.write().await.zone_requests.insert(zone, requested_at);

		let player_id = source.id;
		tokio::spawn(async move {
			sleep(CONFIRMATION_WINDOW).await;

			let Some(player) = SERVER.find_player_by_id(player_id).await
				else { return };

			{
				let mut addon_data = player.addon_data.write().await;
				if addon_data.zone_requests.get(&zone) != Some(&requested_at) {
					return; // client has re-requested a zone update
				}
				addon_data.zone_requests.remove(&zone);
			}

			let distance = player.character.read().await.position.xy().map(|scalar| (scalar / SIZE_ZONE) as i32) - zone;
			if distance.x.abs().max(distance.y.abs()) > RETENTION_RADIUS {
				return; // sending block updates for a zone the client has already "unloaded" can crash it
			}

			player.send_ignoring(&WorldUpdate::from(SERVER.addons.models.blocks_in(zone))).await;
		});
	}
}