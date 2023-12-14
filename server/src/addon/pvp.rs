use std::future::join;
use std::ptr;
use futures::future::join_all;
use tap::{Pipe, Tap};
use protocol::packet::creature_update::CreatureFlag;
use protocol::packet::{CreatureUpdate, StatusEffect, WorldUpdate};
use protocol::packet::common::CreatureId;
use protocol::packet::status_effect::Kind::Affection;
use protocol::utils::flagset::FlagSet;
use crate::server::creature::Creature;

use crate::server::player::Player;
use crate::server::Server;

pub async fn on_creature_update(server: &Server, source: &Player, packet: &CreatureUpdate) -> bool {
	let own_team = source.addon_data.read().await.team;

	let mut packet_copy = packet.clone();
	let Some(ref mut flags_of_copy) = packet_copy.flags
		else { return false; };
	flags_of_copy.set(CreatureFlag::FriendlyFire, true);

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

				let packet_to_send = if is_teammate { packet } else { &packet_copy };
				other_player.send_ignoring(packet_to_send).await;
			};

			Some(future)
		})
		.pipe(join_all)
		.await;

	true
}

pub async fn change_team(server: &Server, player: &Player, new_team: Option<i32>) -> bool {
	let mut addon_data = player.addon_data.write().await;
	if addon_data.team == new_team {
		return false;
	}

	if let Some(old_team) = addon_data.team {
		update_creatures(server, player, old_team, false).await;
	}
	addon_data.team = new_team;
	drop(addon_data); //todo: might be able to drop this even earlier
	if let Some(new_team) = new_team {
		update_creatures(server, player, new_team, true).await;
	}

	true
}

pub fn get_modified_flags(creature: &Creature, friendly_fire: bool) -> Option<FlagSet<u16, CreatureFlag>> {
	creature
		.flags
		.clone()
		.tap_mut(|flags| flags.set(CreatureFlag::FriendlyFire, friendly_fire))
		.pipe(Some)
}

async fn create_flag_update(player: &Player, friendly_fire: bool) -> CreatureUpdate {
	CreatureUpdate {
		id: player.id,
		flags: get_modified_flags(&*player.character.read().await, friendly_fire),
		..Default::default()
	}
}

fn create_heart_update(creature_id: CreatureId, enabled: bool) -> WorldUpdate {
	StatusEffect {
		source: creature_id,
		target: creature_id,
		kind: Affection,
		modifier: 1.0,
		duration: if enabled { i32::MAX } else { 0 },
		creature_id3: CreatureId::default(),
	}.into()
}

async fn update_creatures(server: &Server, player: &Player, team: i32, joined: bool) {
	let flag_update_of_self = create_flag_update(player, !joined).await;
	let heart_update_of_self = create_heart_update(player.id, joined);

	server
		.players
		.read()
		.await
		.iter()
		.filter_map(|other_player| {
			if ptr::eq(other_player.as_ref(), player) {
				return None;
			}

			let future = async {
				if other_player.addon_data.read().await.team != Some(team) {
					return;
				}

				let flag_update_of_other = create_flag_update(other_player, !joined).await;
				let heart_update_of_other = create_heart_update(other_player.id, joined);
				join!(
					player.send_ignoring(&flag_update_of_other),
					player.send_ignoring(&heart_update_of_other),
					other_player.send_ignoring(&flag_update_of_self),
					other_player.send_ignoring(&heart_update_of_self)
				).await;
			};

			Some(future)
		})
		.pipe(join_all)
		.await;
}
