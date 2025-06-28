use std::ptr;
use std::sync::Arc;
use std::future::join;

use futures::future::join_all;
use protocol::packet::{CreatureUpdate, StatusEffect, WorldUpdate};
use protocol::packet::creature_update::Affiliation;
use protocol::packet::status_effect::Kind::Affection;
use protocol::packet::common::CreatureId;
use tap::{Pipe as _, Tap as _};

use crate::server::Server;
use crate::server::player::Player;

use super::map_head;

pub mod display;

pub async fn change_to(server: &Server, player: &Player, new_team: Option<i32>) -> bool {
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
		display::reload(player, &[]).await;
	}

	true
}

fn create_attackability_update(player: &Player, enabled: bool) -> CreatureUpdate {
	CreatureUpdate {
		id: player.id,
		affiliation: Some(if enabled { Affiliation::Enemy } else { Affiliation::Player }),
		rarity: Some(if enabled { 4 } else { 0 }),
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
	let attackability_update_of_self = create_attackability_update(player, !joined);
	let heart_update_of_self = create_heart_update(player.id, joined);
	let map_head_toggle_of_self = map_head::create_toggle_packet(player, !joined);

	server
		.players
		.read()
		.await
		.iter()
		.filter(|other_player| !ptr::eq(other_player.as_ref(), player))
		.map(|other_player| async {
			if other_player.addon_data.read().await.team != Some(team) {
				return;
			}

			let attackability_update_of_other = create_attackability_update(other_player, !joined);
			let heart_update_of_other = create_heart_update(other_player.id, joined);
			let map_head_toggle_of_other = map_head::create_toggle_packet(other_player, !joined);
			join!(
				player.send_ignoring(&attackability_update_of_other),
				player.send_ignoring(&heart_update_of_other),
				player.send_ignoring(&map_head_toggle_of_other),
				other_player.send_ignoring(&attackability_update_of_self),
				other_player.send_ignoring(&heart_update_of_self),
				other_player.send_ignoring(&map_head_toggle_of_self)
			).await;
		})
		.pipe(join_all)
		.await;
}

pub async fn get_members(server: &Server, target_team: i32) -> Vec<Arc<Player>> {
	server
		.players
		.read()
		.await
		.iter()
		.map(|player| async {
			let team = player.addon_data.read().await.team;

			if team != Some(target_team) {
				return None;
			}

			Some(Arc::clone(player))
		})
		.pipe(join_all) //todo: should probably use a stream or sth
		.await
		.into_iter()
		.flatten()
		.collect::<Vec<_>>()
		.tap_mut(|vec| vec.sort_unstable_by_key(|player| player.id.0))
}