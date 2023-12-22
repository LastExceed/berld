use std::ptr;
use futures::future::join_all;
use tap::{Pipe, Tap};
use protocol::packet::creature_update::CreatureFlag;
use protocol::packet::CreatureUpdate;
use protocol::utils::flagset::FlagSet;
use crate::server::creature::Creature;

pub mod team;

use crate::server::player::Player;
use crate::server::Server;

pub async fn on_creature_update(server: &Server, source: &Player, packet: &CreatureUpdate) -> bool {
	team::display::update_for_all_members(server, packet, source).await;
	if packet.flags.is_none() {
		return false;
	};

	let mut packet_with_friendly_fire = packet.clone();
	packet_with_friendly_fire
		.flags
		.as_mut()
		.unwrap()//checked above
		.set(CreatureFlag::FriendlyFire, true);

	let own_team = source.addon_data.read().await.team;

	server
		.players
		.read()
		.await
		.iter()
		.filter_map(|other_player| {
			if ptr::eq(other_player.as_ref(), source) {
				return None;
			}

			let future = async {
				let other_team = other_player.addon_data.read().await.team;
				let is_teammate = own_team.is_some() && own_team == other_team;

				let packet_to_send = if is_teammate { packet } else { &packet_with_friendly_fire };
				other_player.send_ignoring(packet_to_send).await;
			};

			Some(future)
		})
		.pipe(join_all)
		.await;

	true
}

pub fn get_modified_flags(creature: &Creature, friendly_fire: bool) -> FlagSet<u16, CreatureFlag> {
	creature
		.flags
		.clone()
		.tap_mut(|flags| flags.set(CreatureFlag::FriendlyFire, friendly_fire))
}