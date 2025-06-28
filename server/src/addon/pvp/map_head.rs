use std::{sync::Arc, ptr};

use futures::future::join_all;
use protocol::packet::{CreatureUpdate, creature_update::Affiliation, common::CreatureId};
use tap::Pipe as _;

use crate::server::{Server, player::Player, creature::Creature};

pub fn create(character: &Creature, owner_id: CreatureId) -> CreatureUpdate {
    CreatureUpdate {
        id: CreatureId(owner_id.0 + 2500), //todo: claim id from pool
        position: Some(character.position),
        rotation: Some(character.rotation),
        appearance: Some(character.appearance.clone()),
        health: Some(0.0),
        ..Default::default()
    }
}

pub fn create_toggle_packet(source: &Player, enabled: bool) -> CreatureUpdate {
    CreatureUpdate {
        id: CreatureId(source.id.0 + 2500), //todo: claim id from pool
        affiliation: Some(if enabled { Affiliation::Player } else { Affiliation::Neutral }),
        ..Default::default()
    }
}

pub async fn update(server: &Server, source: &Player, packet: &CreatureUpdate, team_members: &[Arc<Player>]) {
	if packet.position.is_none() && packet.rotation.is_none() && packet.appearance.is_none() {
		return;
	}

	let map_head_update = CreatureUpdate {
		id: CreatureId(source.id.0 + 2500),//todo: claim id from pool
		position: packet.position,
		rotation: packet.rotation,
		affiliation: Some(Affiliation::Player),
		appearance: packet.appearance.clone(),
		health: Some(0.0),
		..Default::default()
	};

    server
        .players
        .read()
        .await
        .iter()
        .filter(|player| {
            let is_source = ptr::eq(player.as_ref(), source);
            let is_teammate = team_members
                .iter()
                .any(|member| Arc::ptr_eq(player, member));

            !is_source && !is_teammate
        })
        .map(|player| player.send_ignoring(&map_head_update))
        .pipe(join_all)
        .await;
}