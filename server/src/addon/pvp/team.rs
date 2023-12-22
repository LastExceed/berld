use std::ptr;
use std::future::join;

use futures::future::join_all;
use protocol::packet::{CreatureUpdate, StatusEffect, WorldUpdate};
use protocol::packet::status_effect::Kind::Affection;
use protocol::packet::common::CreatureId;
use tap::Pipe;

use crate::server::Server;
use crate::server::player::Player;

use super::get_modified_flags;

pub mod display;

pub async fn change_team(server: &Server, player: &Player, new_team: Option<i32>) -> bool {
	let mut addon_data = player.addon_data.write().await;
	if addon_data.team == new_team {
		return false;
	}
	let old_value = addon_data.team;
	addon_data.team = new_team;
	drop(addon_data);

	if let Some(old_team) = old_value {
		join!(
			update_creatures(server, player, old_team, false),
			display::reload_for_all_members(server, old_team)
		).await;
	}

	if let Some(new_team) = new_team {
		join!(
			update_creatures(server, player, new_team, true),
			display::reload_for_all_members(server, new_team)
		).await;
	} else {
		display::reload(player, &vec![]).await;
	}

	true
}

async fn create_flag_update(player: &Player, friendly_fire: bool) -> CreatureUpdate {
	CreatureUpdate {
		id: player.id,
		flags: Some(get_modified_flags(&*player.character.read().await, friendly_fire)),
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