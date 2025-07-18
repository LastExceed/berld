use std::{ptr, iter};
use std::sync::Arc;

use futures::future::join_all;
use protocol::packet::CreatureUpdate;
use protocol::packet::creature_update::Occupation;
use protocol::packet::creature_update::{Affiliation, Appearance};
use protocol::packet::common::CreatureId;
use tap::Pipe;

use crate::server::{Server, player::Player, creature_id_pool::CreatureIdPool};

pub fn reserve_dummy_ids(pool: &mut CreatureIdPool) {
	for _ in 1..=3 {
		pool.claim();
	}
}

pub async fn reload_for_all_members(server: &Server, team: i32) {
	let members = super::get_members(server, team).await;

	members
		.iter()
		.map(|recipient| reload(recipient, &members))
		.pipe(join_all)
		.await;
}

pub async fn reload(recipient: &Player, members: &[Arc<Player>]) {
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

pub async fn update_for_all_members(packet: &CreatureUpdate, source: &Player, members: &[Arc<Player>]) {
	let relevant = packet.appearance.is_some()
		|| packet.occupation.is_some()
		|| packet.specialization.is_some()
		|| packet.health.is_some()
		|| packet.multipliers.is_some()
		|| packet.level.is_some()
		|| packet.equipment.is_some()
		|| packet.name.is_some();

	if !relevant {
		return;
	}

	members
		.iter()
		.filter_map(|recipient| {
			if ptr::eq(recipient.as_ref(), source) {
				return None;
			}

			let (id, _) = get_displayed_members(members, recipient)
				.into_iter()
				.find(|(_id, occupant)|
					occupant.is_some_and(|member|
						ptr::eq(member.as_ref(), source)
					)
				)?; //in case [source] is not displayed

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
		occupation: packet.occupation,
		specialization: packet.specialization,
		health: packet.health,
		multipliers: packet.multipliers.clone(),
		level: packet.level,
		equipment: packet.equipment.clone(),
		name: packet.name.clone(),
		..Default::default()
	}
}

fn create_placeholder(id: CreatureId) -> CreatureUpdate {
	CreatureUpdate {
		id,
		affiliation: Some(Affiliation::Player),
		appearance: Some(Appearance { head_model: -1, hair_model: -1, ..Default::default() }), //todo: default for Appearance?,
		occupation: Some(Occupation::None),
		health: Some(0.0),
		multipliers: Some(Default::default()),
		level: Some(0),
		equipment: Some(Default::default()),
		name: Some("".into()),
		..Default::default()
	}
}

fn get_displayed_members<'team>(members: &'team [Arc<Player>], pov: &Player) -> [(CreatureId, Option<&'team Arc<Player>>); 3] {
    members
        .iter()
        .filter(|other| !ptr::eq(pov, other.as_ref()))
		.map(Some)
		.chain(iter::repeat(None))
		.take(3)
		.enumerate()
		.map(|(i, member)| (CreatureId(i as i64 + 1), member))
		.collect::<Vec<_>>()
		.try_into()
		.unwrap()
}