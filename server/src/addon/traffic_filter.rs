use protocol::packet::creature_update::CreatureFlag;
use protocol::packet::CreatureUpdate;

use crate::server::creature::Creature;

pub fn filter(packet: &mut CreatureUpdate, former_state: &Creature, updated_state: &Creature) -> bool {
	packet.rotation       = None;//this would be useful if it worked as intended, but unfortunately it has no effect
	packet.head_tilt      = None;
	packet.flags_physics  = None;
	packet.combo_timeout  = None;
	packet.mana           = None;
	packet.blocking_gauge = None;
	packet.experience     = None;
	packet.home_zone      = None;
	packet.home           = None;
	packet.zone_to_reveal = None;
	packet.mana_cubes     = None;
	//always keep:
	//- affiliation
	//- race
	//- animation
	//- appearance
	//- flags
	//- classMajor
	//- classMinor
	//- health
	//- multipliers
	//- level
	//- consumable
	//- equipment
	//- name
	//- rarity
	//- skilltree //need this because receiving clients locally remove the glider if the skill isnt learned

	//todo:
	//- position
	//- velocity
	//- acceleration
	//- combo
	//- showPatchtime
	//- manaCharge
	//- unknown24
	//- unknown25
	//- unknown31
	//- unknown21
	//- master
	//- unknown36
	//- unknown38
	//- unknown42

//	let need_velocity_z = packet.velocity.map_or(false, |velocity| {
//		if updated_state.flags.get(CreatureFlag::Climbing) {
//			false
//		} else if updated_state.flags_physics.get(PhysicsFlag::Swimming) {
//			velocity.z > 1.0 && velocity.z - (updated_state.acceleration.z / 80.0 * 12.0) > 1.0 //wip
//		} else if velocity.z < former_state.velocity.z {
//			false
//		} else if updated_state.flags_physics.get(PhysicsFlag::OnGround) {
//			velocity.z > 0.0
//		} else { //airborne
//			true
//		}
//	});
//	let glider_hovering = need_velocity_z && updated_state.flags.get(CreatureFlag::Gliding);
//	let movement_changed = packet.acceleration.map_or(false, |acceleration| acceleration.metric_distance(&former_state.acceleration) > 0f32);//todo: compare to last sent (4)
//	let teleported = packet.position.map_or(false, |position| distance::<f64, 3>(&position.cast(), &former_state.position.cast()) > SIZE_BLOCK as f64 * 4.0);
//	let dodge_started = packet.effect_time_dodge.map_or(false, |effect_time_dodge| effect_time_dodge > former_state.effect_time_dodge);
//	let intercepting = updated_state.animation == Animation::Intercept;

//	if !movement_changed {
//		packet.acceleration = None;
//		if !glider_hovering && !teleported {
//			packet.position = None;
//		}
//	}
//	if !need_velocity_z && !dodge_started && !intercepting {
//		packet.velocity = None;
//	}

	let new_animation_started = updated_state.animation_time < former_state.animation_time;

	packet.animation_time.filter_in_place(|_| new_animation_started);

	packet.velocity_extra.filter_in_place(|velocity_extra| {
		former_state.velocity_extra
			.into_iter()
			.zip(velocity_extra)
			.any(|(old, new)| !(0.0..1.0).contains(&(new / old)))//todo: there gotta be a better way to do this
	});

	packet.effect_time_dodge.filter_in_place(|value| *value > former_state.effect_time_dodge);
	packet.effect_time_stun .filter_in_place(|value| *value > former_state.effect_time_stun );
	packet.effect_time_fear .filter_in_place(|value| *value > former_state.effect_time_fear );
	packet.effect_time_chill.filter_in_place(|value| *value > former_state.effect_time_chill);
	packet.effect_time_wind .filter_in_place(|value| *value > former_state.effect_time_wind );

	//there is a bug in the game where starting a new animation for a foreign dodging creature doesn't cancel their dodge roll
	//this normally stays unnoticed as the creature will eventually report the end of their dodge roll on their own,
	//and the next report of to their animation time then starts the animation.
	//but since we filter out all timer updates that that just reflect the natural passage of time, we now need to cancel the dodge manually
	if new_animation_started && former_state.effect_time_dodge != 0 {
		packet.effect_time_dodge = Some(0);
	}

	packet.aim_offset.filter_in_place(|_| updated_state.flags.get(CreatureFlag::Aiming));//todo: compare to last sent (2)

	//todo: macro
	packet.position          .is_some() ||
	packet.rotation          .is_some() ||
	packet.velocity          .is_some() ||
	packet.acceleration      .is_some() ||
	packet.velocity_extra    .is_some() ||
	packet.head_tilt         .is_some() ||
	packet.flags_physics     .is_some() ||
	packet.affiliation       .is_some() ||
	packet.race              .is_some() ||
	packet.animation         .is_some() ||
	packet.animation_time    .is_some() ||
	packet.combo             .is_some() ||
	packet.combo_timeout     .is_some() ||
	packet.appearance        .is_some() ||
	packet.flags             .is_some() ||
	packet.effect_time_dodge .is_some() ||
	packet.effect_time_stun  .is_some() ||
	packet.effect_time_fear  .is_some() ||
	packet.effect_time_chill .is_some() ||
	packet.effect_time_wind  .is_some() ||
	packet.show_patch_time   .is_some() ||
	packet.occupation        .is_some() ||
	packet.specialization    .is_some() ||
	packet.mana_charge       .is_some() ||
	packet.unknown24         .is_some() ||
	packet.unknown25         .is_some() ||
	packet.aim_offset        .is_some() ||
	packet.health            .is_some() ||
	packet.mana              .is_some() ||
	packet.blocking_gauge    .is_some() ||
	packet.multipliers       .is_some() ||
	packet.unknown31         .is_some() ||
	packet.unknown32         .is_some() ||
	packet.level             .is_some() ||
	packet.experience        .is_some() ||
	packet.master            .is_some() ||
	packet.unknown36         .is_some() ||
	packet.rarity            .is_some() ||
	packet.unknown38         .is_some() ||
	packet.home_zone         .is_some() ||
	packet.home              .is_some() ||
	packet.zone_to_reveal    .is_some() ||
	packet.unknown42         .is_some() ||
	packet.consumable        .is_some() ||
	packet.equipment         .is_some() ||
	packet.name              .is_some() ||
	packet.skill_tree        .is_some() ||
	packet.mana_cubes        .is_some()
	//returns whether any data is remaining
}

trait FilterInPlace<T> {
	fn filter_in_place<P: FnOnce(&T) -> bool>(&mut self, predicate: P);
}

impl<T> FilterInPlace<T> for Option<T> {
	fn filter_in_place<P: FnOnce(&T) -> bool>(&mut self, predicate: P) {
		if let Some(value) = self && !predicate(value) {
			*self = None;
		}
	}
}