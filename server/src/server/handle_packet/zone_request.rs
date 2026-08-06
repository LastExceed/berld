use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::time::sleep;

use protocol::nalgebra::{Point2, Vector2};
use protocol::packet::{AreaRequest, WorldUpdate};
use protocol::packet::area_request::Zone;
use protocol::packet::common::CreatureId;
use protocol::utils::zone_of;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::addon_data::ZoneState;
use crate::server::player::{Player, ZONE_DATA_RADIUS, ZONE_RETENTION_RADIUS};
use crate::server::Server;
use crate::SERVER;

const CENTER_SETTLE: Duration = Duration::from_secs(1);
const NEIGHBOR_SETTLE: Duration = Duration::from_secs(3);
const STALE_RETRY_GRACE: Duration = Duration::from_secs(2);
const CENTER_TIMEOUT: Duration = Duration::from_secs(15);
const CENTER_POLL: Duration = Duration::from_millis(200);

impl HandlePacket<AreaRequest<Zone>> for Server {
	async fn handle_packet(&self, source: &Player, packet: AreaRequest<Zone>) {
		let zone = packet.0;

		let Some(player) = self.find_player_by_id(source.id).await
		else { return };

		self.answer_request(&player, zone).await;
		self.schedule_neighborhood_reveal(source.id, zone);
	}
}

impl Server {
	// client's zone request proves lack of content, no matter what the server believes
	// it always gets answered, otherwise client re-requests forever
	async fn answer_request(&self, player: &Arc<Player>, zone: Point2<i32>) {
		{
			let mut addon_data = player.addon_data.write().await;
			match addon_data.zone_states.get(&zone) {
				Some(ZoneState::Pending) => return,
				Some(ZoneState::Revealed(at)) if at.elapsed() < STALE_RETRY_GRACE => return,
				_ => {}
			}
			// claimed in the same lock as the check
			addon_data.zone_states.insert(zone, ZoneState::Pending);
		}

		// loot alone is safe immediately
		let settle = if self.addons.models.packet_for(zone).is_some() {
			CENTER_SETTLE
		} else {
			Duration::ZERO
		};

		let player = Arc::clone(player);
		tokio::spawn(async move {
			SERVER.deliver_zone(&player, zone, settle).await;
		});
	}

	async fn reveal_neighborhood(&self, player: &Arc<Player>, center: Point2<i32>) {
		for zone in neighbors(center) {
			let settled_in = zone_of(player.character.read().await.position);
			if settled_in != center {
				return;
			}

			if !self.has_content(zone).await {
				continue;
			}

			{
				let mut addon_data = player.addon_data.write().await;
				if addon_data.zone_states.contains_key(&zone) {
					continue;
				}
				addon_data.zone_states.insert(zone, ZoneState::Pending);
			}

			self.deliver_zone(player, zone, NEIGHBOR_SETTLE).await;
		}
	}

	// everything waits out the settle, not just terrain: acknowledgment itself is zone-keyed
	// client ignores zone update if sent too early (before client-side zone generation)
	async fn deliver_zone(&self, player: &Arc<Player>, zone: Point2<i32>, settle: Duration) {
		sleep(settle).await;

		let delivered = self.send_zone(player, zone).await;

		let mut addon_data = player.addon_data.write().await;
		if delivered {
			addon_data.zone_states.insert(zone, ZoneState::Revealed(Instant::now()));
		} else {
			// let a later reveal or request retry it
			addon_data.zone_states.remove(&zone);
		}
	}

	async fn send_zone(&self, player: &Arc<Player>, zone: Point2<i32>) -> bool {
		// a zone change that took this zone out of range drops the claim
		let still_claimed = matches!(
			player.addon_data.read().await.zone_states.get(&zone),
			Some(ZoneState::Pending)
		);
		if !still_claimed || !player.is_near(zone).await {
			return false;
		}

		if let Some(blocks) = self.addons.models.packet_for(zone)
			&& player.send_raw(&blocks).await.is_err()
		{
			return false;
		}

		self.acknowledge(player, zone).await;
		true
	}

	// an entry for a zone (even an empty one) acknowledges discovery
	// and stops the client from re-requesting zones that remain loaded client-side
	// p48 is unsafe, so empty loot updates are sent for empty zones instead
	async fn acknowledge(&self, player: &Player, zone: Point2<i32>) {
		let loot_in_zone = self.loot
			.read().await
			.get(&zone)
			.cloned()
			.unwrap_or_default();

		let acknowledgment = WorldUpdate {
			loot: [(zone, loot_in_zone)].into(),
			..Default::default()
		};
		player.send_ignoring(&acknowledgment).await;
	}

	async fn center_ready(&self, player: &Arc<Player>, center: Point2<i32>) -> bool {
		if self.addons.models.packet_for(center).is_none() {
			return true;
		}

		let deadline = Instant::now() + CENTER_TIMEOUT;

		while Instant::now() < deadline {
			let state = player.addon_data.read().await.zone_states.get(&center).copied();

			match state {
				Some(ZoneState::Revealed(_)) => return true,
				// claim was dropped; player left range
				None => return false,
				_ => sleep(CENTER_POLL).await
			}
		}

		false
	}

	async fn has_content(&self, zone: Point2<i32>) -> bool {
		self.addons.models.packet_for(zone).is_some()
			|| self.loot.read().await.contains_key(&zone)
	}

	pub async fn prune_zone_states(&self, player: &Player, center: Point2<i32>) {
		player
			.addon_data
			.write()
			.await
			.zone_states
			.retain(|zone, state| match state {
				ZoneState::Awaited(_) | ZoneState::Pending =>
					chebyshev_distance(*zone, center) <= ZONE_DATA_RADIUS,
				ZoneState::Revealed(_) =>
					chebyshev_distance(*zone, center) <= ZONE_RETENTION_RADIUS
			});
	}

	pub fn schedule_neighborhood_reveal(&self, player_id: CreatureId, center: Point2<i32>) {
		tokio::spawn(async move {
			let Some(player) = SERVER.find_player_by_id(player_id).await
			else { return };

			SERVER.prune_zone_states(&player, center).await;

			if SERVER.addons.models.packet_for(center).is_some() {
				player
					.addon_data
					.write()
					.await
					.zone_states
					.entry(center)
					.or_insert(ZoneState::Awaited(Instant::now()));
			}

			if !SERVER.center_ready(&player, center).await {
				return;
			}

			{
				let mut addon_data = player.addon_data.write().await;
				if addon_data.revealing_neighborhood {
					return;
				}
				addon_data.revealing_neighborhood = true;
			}

			SERVER.reveal_neighborhood(&player, center).await;

			player.addon_data.write().await.revealing_neighborhood = false;
		});
	}
}

fn neighbors(center: Point2<i32>) -> [Point2<i32>; 8] {
	[
		center + Vector2::new( 0, -1),
		center + Vector2::new( 0,  1),
		center + Vector2::new(-1,  0),
		center + Vector2::new( 1,  0),
		center + Vector2::new(-1, -1),
		center + Vector2::new(-1,  1),
		center + Vector2::new( 1, -1),
		center + Vector2::new( 1,  1)
	]
}

fn chebyshev_distance(from: Point2<i32>, to: Point2<i32>) -> i32 {
	let delta = from - to;

	delta.x.abs().max(delta.y.abs())
}