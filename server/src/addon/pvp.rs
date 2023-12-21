use std::{future::join, sync::Arc, convert::identity, iter};
use std::ptr;
use futures::future::join_all;
use tap::{Pipe, Tap};
use protocol::packet::creature_update::{CreatureFlag, Affiliation, Appearance};
use protocol::packet::{CreatureUpdate, StatusEffect, WorldUpdate};
use protocol::packet::common::CreatureId;
use protocol::packet::status_effect::Kind::Affection;
use protocol::utils::flagset::FlagSet;
use crate::server::creature::Creature;

use crate::server::player::Player;
use crate::server::Server;
use crate::server::creature_id_pool::CreatureIdPool;

pub async fn on_creature_update(server: &Server, source: &Player, packet: &CreatureUpdate) -> bool {
	update_team_displays(server, packet, source).await;
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
			reload_all_team_displays(server, old_team)
		).await;
	}

	if let Some(new_team) = new_team {
		join!(
			update_creatures(server, player, new_team, true),
			reload_all_team_displays(server, new_team)
		).await;
	} else {
		reload_team_display(player, &vec![]).await;
	}

	true
}

pub fn reserve_team_display_dummies(pool: &mut CreatureIdPool) {
	for _ in 1..=3 {
		pool.claim();
	}
}

pub fn get_modified_flags(creature: &Creature, friendly_fire: bool) -> FlagSet<u16, CreatureFlag> {
	creature
		.flags
		.clone()
		.tap_mut(|flags| flags.set(CreatureFlag::FriendlyFire, friendly_fire))
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

async fn reload_all_team_displays(server: &Server, team: i32) {
	let members = get_all_members(server, team).await;

	members
		.iter()
		.map(|recipient| reload_team_display(recipient, &members))
		.pipe(join_all)
		.await;
}

pub async fn reload_team_display(recipient: &Player, members: &Vec<Arc<Player>>) {
	get_displayed(members, recipient)
		.map(|(dummy_id, occupant)| reload_display_slot(recipient, dummy_id, occupant))
		.pipe(join_all)
		.await;
}

async fn reload_display_slot(pov: &Player, dummy_id: CreatureId, occupant: Option<&Arc<Player>>) {
	let display_update = if let Some(member) = occupant {
		member
			.character
			.read()
			.await
			.to_update(dummy_id)//technically incorrect, but irrelevant
			.pipe(|packet| create_display_update(&packet, dummy_id))
	} else {
		create_placeholder(dummy_id)
	};

	pov.send_ignoring(&display_update).await;
}

async fn update_team_displays(server: &Server, packet: &CreatureUpdate, source: &Player) {
	let relevant = packet.health.is_some()
		|| packet.appearance.is_some()
		|| packet.name.is_some();

	if !relevant {
		return;
	}

	let Some(target_team) = source.addon_data.read().await.team
		else { return; };

	let members = get_all_members(server, target_team).await;

	members
		.iter()
		.filter_map(|recipient| {
			if ptr::eq(recipient.as_ref(), source) {
				return None;
			}

			let Some((id, _)) = get_displayed(&members, recipient)
				.into_iter()
				.find(|(_id, member)|
					member.is_some_and(|member|
						ptr::eq(member.as_ref(), source)
					)
				)
				else { return None; }; //in case [source] is not displayed

			let future = async move {
				let display_update = create_display_update(packet, id);
				recipient.send_ignoring(&display_update).await;
			};

			Some(future)
		})
		.pipe(join_all)
		.await;
}

fn create_display_update(packet: &CreatureUpdate, id: CreatureId) -> CreatureUpdate {
	CreatureUpdate {
		id,
		appearance: packet.appearance.clone(),
		health: packet.health,
		name: packet.name.clone(),
		..Default::default()
	}
}

fn create_placeholder(id: CreatureId) -> CreatureUpdate {
	CreatureUpdate {
		id,
		affiliation: Some(Affiliation::Player),
		appearance: Some(Appearance { head_model: -1, hair_model: -1, ..Default::default() }), //todo: default for Appearance?,
		health: Some(0.0),
		name: Some("".into()),
		..Default::default()
	}
}

fn get_displayed<'team>(members: &'team Vec<Arc<Player>>, pov: &Player) -> [(CreatureId, Option<&'team Arc<Player>>); 3] {
    members
        .iter()
        .filter(|other| !ptr::eq(pov, other.as_ref()))
		.map(|other| Some(other))
		.chain(iter::repeat(None))
		.take(3)
		.enumerate()
		.map(|(i, member)| (CreatureId(i as i64 + 1), member))
		.collect::<Vec<_>>()
		.try_into()
		.unwrap()
}

async fn get_all_members(server: &Server, target_team: i32) -> Vec<Arc<Player>> {
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
		.filter_map(identity)
		.collect::<Vec<_>>()
		.tap_mut(|vec| vec.sort_unstable_by_key(|player| player.id.0))
}