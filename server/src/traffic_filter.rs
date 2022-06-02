use protocol::packet::creature_update::CreatureUpdate;
use crate::player::Player;

pub fn filter(creature_update: &mut CreatureUpdate, _source: &Player) -> bool {
	// position
	creature_update.rotation = None;
	// velocity
	// acceleration
	// velocityExtra
	creature_update.climb_animation_state = None;
	creature_update.flags_physics = None;
	//affiliation
	//race
	//animation
	// animationTime
	//combo
	creature_update.hit_time_out = None;
	//appearance
	//flags
	// effectTimeDodge
	// effectTimeStun
	// effectTimeFear
	// effectTimeIce
	// effectTimeWind
	//showPatchtime
	//classMajor
	//classMinor
	//manaCharge
	//unknown24
	//unknown25
	// aimDisplacement
	//health
	creature_update.mana = None;
	creature_update.blocking_gauge = None;
	//multipliers
	//unknown31
	//unknown21
	//level
	creature_update.experience = None;
	//master
	//unknown36
	//powerBase
	//unknown38
	creature_update.home_chunk = None;
	creature_update.home = None;
	creature_update.chunk_to_reveal = None;
	//unknown42
	//consumable
	//equipment
	//name
	creature_update.skill_tree = None;
	creature_update.mana_cubes = None;

	//todo: macro
	let mut any_data_remaining = false;
	any_data_remaining |= creature_update.position             .is_some();
	any_data_remaining |= creature_update.rotation             .is_some();
	any_data_remaining |= creature_update.velocity             .is_some();
	any_data_remaining |= creature_update.acceleration         .is_some();
	any_data_remaining |= creature_update.velocity_extra       .is_some();
	any_data_remaining |= creature_update.climb_animation_state.is_some();
	any_data_remaining |= creature_update.flags_physics        .is_some();
	any_data_remaining |= creature_update.affiliation          .is_some();
	any_data_remaining |= creature_update.race                 .is_some();
	any_data_remaining |= creature_update.animation            .is_some();
	any_data_remaining |= creature_update.animation_time       .is_some();
	any_data_remaining |= creature_update.combo                .is_some();
	any_data_remaining |= creature_update.hit_time_out         .is_some();
	any_data_remaining |= creature_update.appearance           .is_some();
	any_data_remaining |= creature_update.flags                .is_some();
	any_data_remaining |= creature_update.effect_time_dodge    .is_some();
	any_data_remaining |= creature_update.effect_time_stun     .is_some();
	any_data_remaining |= creature_update.effect_time_fear     .is_some();
	any_data_remaining |= creature_update.effect_time_ice      .is_some();
	any_data_remaining |= creature_update.effect_time_wind     .is_some();
	any_data_remaining |= creature_update.show_patch_time      .is_some();
	any_data_remaining |= creature_update.combat_class_major   .is_some();
	any_data_remaining |= creature_update.combat_class_minor   .is_some();
	any_data_remaining |= creature_update.mana_charge          .is_some();
	any_data_remaining |= creature_update.unknown24            .is_some();
	any_data_remaining |= creature_update.unknown25            .is_some();
	any_data_remaining |= creature_update.aim_displacement     .is_some();
	any_data_remaining |= creature_update.health               .is_some();
	any_data_remaining |= creature_update.mana                 .is_some();
	any_data_remaining |= creature_update.blocking_gauge       .is_some();
	any_data_remaining |= creature_update.multipliers          .is_some();
	any_data_remaining |= creature_update.unknown31            .is_some();
	any_data_remaining |= creature_update.unknown32            .is_some();
	any_data_remaining |= creature_update.level                .is_some();
	any_data_remaining |= creature_update.experience           .is_some();
	any_data_remaining |= creature_update.master               .is_some();
	any_data_remaining |= creature_update.unknown36            .is_some();
	any_data_remaining |= creature_update.power_base           .is_some();
	any_data_remaining |= creature_update.unknown38            .is_some();
	any_data_remaining |= creature_update.home_chunk           .is_some();
	any_data_remaining |= creature_update.home                 .is_some();
	any_data_remaining |= creature_update.chunk_to_reveal      .is_some();
	any_data_remaining |= creature_update.unknown42            .is_some();
	any_data_remaining |= creature_update.consumable           .is_some();
	any_data_remaining |= creature_update.equipment            .is_some();
	any_data_remaining |= creature_update.name                 .is_some();
	any_data_remaining |= creature_update.skill_tree           .is_some();
	any_data_remaining |= creature_update.mana_cubes           .is_some();

	any_data_remaining
}