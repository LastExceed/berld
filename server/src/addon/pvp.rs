use std::ptr;
use futures::future::join_all;
use tap::Pipe as _;
use protocol::packet::creature_update::Affiliation;
use protocol::packet::CreatureUpdate;

pub mod team;
pub mod map_head;

use crate::server::player::Player;
use crate::server::Server;

pub async fn on_creature_update(server: &Server, source: &Player, packet: &CreatureUpdate) {
	let team_members =
		if let Some(target_team) = source.addon_data.read().await.team {
			team::get_members(server, target_team).await
		} else {
			vec![]
		};
	team::display::update_for_all_members(packet, source, &team_members).await;
	map_head::update(server, source, packet, &team_members).await;
}

pub async fn broadcast(server: &Server, source: &Player, packet: &CreatureUpdate) -> bool {
	if packet.affiliation.is_none() && packet.rarity.is_none() {//if packet.flags.is_none() {
		return false;
	}

	let mut pvp_enabled_packet = packet.clone();

	if let Some(ref mut affiliation) = pvp_enabled_packet.affiliation {
		*affiliation = Affiliation::Enemy;
	}

	if let Some(ref mut rarity) = pvp_enabled_packet.rarity {
		*rarity = 4;
	}

	let own_team = source.addon_data.read().await.team;

	server
		.players
		.read()
		.await
		.iter()
		.filter(|target| !ptr::eq(target.as_ref(), source))
		.map(|target| async {
			let other_team = target.addon_data.read().await.team;
			let is_teammate = own_team.is_some() && own_team == other_team;

			let packet_to_send = if is_teammate { packet } else { &pvp_enabled_packet };
			target.send_ignoring(packet_to_send).await;
		})
		.pipe(join_all)
		.await;

	true
}