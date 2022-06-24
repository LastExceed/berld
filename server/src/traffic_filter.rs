use protocol::packet::creature_update::{CreatureFlag, PhysicsFlag};
use protocol::packet::CreatureUpdate;

use crate::creature::Creature;

pub fn filter(packet: &mut CreatureUpdate, previous: &Creature, current: &Creature) -> bool {
	packet.rotation = None;
	packet.climb_animation_state = None;
	packet.flags_physics = None;
	packet.hit_time_out = None;
	packet.mana = None;
	packet.blocking_gauge = None;
	packet.experience = None;
	packet.home_zone = None;
	packet.home = None;
	packet.zone_to_reveal = None;
	packet.skill_tree = None;
	packet.mana_cubes = None;
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

	//todo:
	//- combo
	//- showPatchtime
	//- manaCharge
	//- unknown24
	//- unknown25
	//- unknown31
	//- unknown21
	//- master
	//- unknown36
	//- powerBase
	//- unknown38
	//- unknown42

	//x and y are always overridden by acceleration
	let need_velocity_z = packet.velocity.map_or(false, |velocity| {
		if current.flags.get(CreatureFlag::Climbing) {
			false
		} else if current.flags_physics.get(PhysicsFlag::Swimming) {
			velocity.z > 1f32 && velocity.z - (current.acceleration.z / 80f32 * 12f32) > 1f32 //wip
		} else if velocity.z < previous.velocity.z {
			false
		} else if current.flags_physics.get(PhysicsFlag::OnGround) {
			velocity.z > 0f32
		} else { //airborne
			true
		}
	});
	let glider_hovering = need_velocity_z && current.flags.get(CreatureFlag::Gliding);
	let movement_changed = packet.acceleration.map_or(false, |acceleration| acceleration.metric_distance(&previous.acceleration) > 0f32);//todo: compare to last sent (4)
	let new_animation_started = packet.animation_time.map_or(false, |animation_time| animation_time < previous.animation_time);

	if !movement_changed {
		packet.acceleration = None;
		if !glider_hovering {
			packet.position = None;
		}
	}
	if !need_velocity_z {
		packet.velocity = None;
	}

	if !new_animation_started {
		packet.animation_time = None;
	}

	packet.velocity_extra = packet.velocity_extra.filter(|velocity_extra| {
		velocity_extra
			.iter()
			.zip(previous.velocity_extra.iter())
			.any(|(new, old)| !(0f32..1f32).contains(&(new / old)))//todo: there gotta be a better way to do this
	});

	packet.effect_time_dodge = packet.effect_time_dodge.filter(|value| *value > previous.effect_time_dodge);
	packet.effect_time_stun  = packet.effect_time_stun .filter(|value| *value > previous.effect_time_stun );
	packet.effect_time_fear  = packet.effect_time_fear .filter(|value| *value > previous.effect_time_fear );
	packet.effect_time_chill = packet.effect_time_chill.filter(|value| *value > previous.effect_time_chill);
	packet.effect_time_wind  = packet.effect_time_wind .filter(|value| *value > previous.effect_time_wind );

	packet.aim_offset = packet.aim_offset.filter(|_| current.flags.get(CreatureFlag::Aiming));//todo: compare to last sent (2)



	//todo: macro
	packet.position             .is_some() ||
	packet.rotation             .is_some() ||
	packet.velocity             .is_some() ||
	packet.acceleration         .is_some() ||
	packet.velocity_extra       .is_some() ||
	packet.climb_animation_state.is_some() ||
	packet.flags_physics        .is_some() ||
	packet.affiliation          .is_some() ||
	packet.race                 .is_some() ||
	packet.animation            .is_some() ||
	packet.animation_time       .is_some() ||
	packet.combo                .is_some() ||
	packet.hit_time_out         .is_some() ||
	packet.appearance           .is_some() ||
	packet.flags                .is_some() ||
	packet.effect_time_dodge    .is_some() ||
	packet.effect_time_stun     .is_some() ||
	packet.effect_time_fear     .is_some() ||
	packet.effect_time_chill    .is_some() ||
	packet.effect_time_wind     .is_some() ||
	packet.show_patch_time      .is_some() ||
	packet.combat_class_major   .is_some() ||
	packet.combat_class_minor   .is_some() ||
	packet.mana_charge          .is_some() ||
	packet.unknown24            .is_some() ||
	packet.unknown25            .is_some() ||
	packet.aim_offset           .is_some() ||
	packet.health               .is_some() ||
	packet.mana                 .is_some() ||
	packet.blocking_gauge       .is_some() ||
	packet.multipliers          .is_some() ||
	packet.unknown31            .is_some() ||
	packet.unknown32            .is_some() ||
	packet.level                .is_some() ||
	packet.experience           .is_some() ||
	packet.master               .is_some() ||
	packet.unknown36            .is_some() ||
	packet.power_base           .is_some() ||
	packet.unknown38            .is_some() ||
	packet.home_zone            .is_some() ||
	packet.home                 .is_some() ||
	packet.zone_to_reveal       .is_some() ||
	packet.unknown42            .is_some() ||
	packet.consumable           .is_some() ||
	packet.equipment            .is_some() ||
	packet.name                 .is_some() ||
	packet.skill_tree           .is_some() ||
	packet.mana_cubes           .is_some()
	//returns whether any data is remaining
}