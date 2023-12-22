use std::{ptr, iter};
use std::convert::identity;
use std::sync::Arc;

use futures::future::join_all;
use protocol::packet::CreatureUpdate;
use protocol::packet::creature_update::{Affiliation, Appearance};
use protocol::packet::common::CreatureId;
use tap::{Pipe, Tap};

use crate::server::{Server, player::Player, creature_id_pool::CreatureIdPool};

pub fn reserve_dummy_ids(pool: &mut CreatureIdPool) {
	for _ in 1..=3 {
		pool.claim();
	}
}

pub async fn reload_for_all_members(server: &Server, team: i32) {
	let members = get_all_members(server, team).await;

	members
		.iter()
		.map(|recipient| reload(recipient, &members))
		.pipe(join_all)
		.await;
}

pub async fn reload(recipient: &Player, members: &Vec<Arc<Player>>) {
	get_displayed_members(members, recipient)
		.map(|(dummy_id, occupant)| reload_slot(recipient, dummy_id, occupant))
		.pipe(join_all)
		.await;
}

async fn reload_slot(pov: &Player, dummy_id: CreatureId, occupant: Option<&Arc<Player>>) {
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

pub async fn update_for_all_members(server: &Server, packet: &CreatureUpdate, source: &Player) {
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

			let Some((id, _)) = get_displayed_members(&members, recipient)
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

fn get_displayed_members<'team>(members: &'team Vec<Arc<Player>>, pov: &Player) -> [(CreatureId, Option<&'team Arc<Player>>); 3] {
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