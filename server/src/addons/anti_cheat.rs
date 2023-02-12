use std::fmt::Debug;
use std::ops::RangeBounds;
use std::result;

use boolinator::Boolinator;

use protocol::packet::CreatureUpdate;

use crate::server::creature::Creature;

use self::creature_update::*;

pub mod creature_update;

pub fn inspect_creature_update(packet: &CreatureUpdate, former_state: &Creature, updated_state: &Creature) -> Result {
	//todo: macro
	packet.position          .map_or_ok(|value| inspect_position(          value, &former_state, &updated_state))?;
	packet.rotation          .map_or_ok(|value| inspect_rotation(          value, &former_state, &updated_state))?;
	packet.velocity          .map_or_ok(|value| inspect_velocity(          value, &former_state, &updated_state))?;
	packet.acceleration      .map_or_ok(|value| inspect_acceleration(      value, &former_state, &updated_state))?;
	packet.velocity_extra    .map_or_ok(|value| inspect_velocity_extra(    value, &former_state, &updated_state))?;
	packet.head_tilt         .map_or_ok(|value| inspect_head_tilt(         value, &former_state, &updated_state))?;
	packet.flags_physics     .map_or_ok(|value| inspect_flags_physics(     value, &former_state, &updated_state))?;
	packet.affiliation       .map_or_ok(|value| inspect_affiliation(       value, &former_state, &updated_state))?;
	packet.race              .map_or_ok(|value| inspect_race(              value, &former_state, &updated_state))?;
	packet.animation         .map_or_ok(|value| inspect_animation(         value, &former_state, &updated_state))?;
	packet.animation_time    .map_or_ok(|value| inspect_animation_time(    value, &former_state, &updated_state))?;
	packet.combo             .map_or_ok(|value| inspect_combo(             value, &former_state, &updated_state))?;
	packet.combo_timeout     .map_or_ok(|value| inspect_combo_timeout(     value, &former_state, &updated_state))?;
	packet.appearance        .map_or_ok(|value| inspect_appearance(        value, &former_state, &updated_state))?;
	packet.flags             .map_or_ok(|value| inspect_flags(             value, &former_state, &updated_state))?;
	packet.effect_time_dodge .map_or_ok(|value| inspect_effect_time_dodge( value, &former_state, &updated_state))?;
	packet.effect_time_stun  .map_or_ok(|value| inspect_effect_time_stun(  value, &former_state, &updated_state))?;
	packet.effect_time_fear  .map_or_ok(|value| inspect_effect_time_fear(  value, &former_state, &updated_state))?;
	packet.effect_time_chill .map_or_ok(|value| inspect_effect_time_chill( value, &former_state, &updated_state))?;
	packet.effect_time_wind  .map_or_ok(|value| inspect_effect_time_wind(  value, &former_state, &updated_state))?;
	packet.show_patch_time   .map_or_ok(|value| inspect_show_patch_time(   value, &former_state, &updated_state))?;
	packet.occupation        .map_or_ok(|value| inspect_occupation(        value, &former_state, &updated_state))?;
	packet.specialization    .map_or_ok(|value| inspect_specialization(    value, &former_state, &updated_state))?;
	packet.mana_charge       .map_or_ok(|value| inspect_mana_charge(       value, &former_state, &updated_state))?;
	packet.unknown24         .map_or_ok(|value| inspect_unknown24(         value, &former_state, &updated_state))?;
	packet.unknown25         .map_or_ok(|value| inspect_unknown25(         value, &former_state, &updated_state))?;
	packet.aim_offset        .map_or_ok(|value| inspect_aim_offset(        value, &former_state, &updated_state))?;
	packet.health            .map_or_ok(|value| inspect_health(            value, &former_state, &updated_state))?;
	packet.mana              .map_or_ok(|value| inspect_mana(              value, &former_state, &updated_state))?;
	packet.blocking_gauge    .map_or_ok(|value| inspect_blocking_gauge(    value, &former_state, &updated_state))?;
	packet.multipliers       .map_or_ok(|value| inspect_multipliers(       value, &former_state, &updated_state))?;
	packet.unknown31         .map_or_ok(|value| inspect_unknown31(         value, &former_state, &updated_state))?;
	packet.unknown32         .map_or_ok(|value| inspect_unknown32(         value, &former_state, &updated_state))?;
	packet.level             .map_or_ok(|value| inspect_level(             value, &former_state, &updated_state))?;
	packet.experience        .map_or_ok(|value| inspect_experience(        value, &former_state, &updated_state))?;
	packet.master            .map_or_ok(|value| inspect_master(            value, &former_state, &updated_state))?;
	packet.unknown36         .map_or_ok(|value| inspect_unknown36(         value, &former_state, &updated_state))?;
	packet.rarity            .map_or_ok(|value| inspect_rarity(            value, &former_state, &updated_state))?;
	packet.unknown38         .map_or_ok(|value| inspect_unknown38(         value, &former_state, &updated_state))?;
	packet.home_zone         .map_or_ok(|value| inspect_home_zone(         value, &former_state, &updated_state))?;
	packet.home              .map_or_ok(|value| inspect_home(              value, &former_state, &updated_state))?;
	packet.zone_to_reveal    .map_or_ok(|value| inspect_zone_to_reveal(    value, &former_state, &updated_state))?;
	packet.unknown42         .map_or_ok(|value| inspect_unknown42(         value, &former_state, &updated_state))?;
	packet.consumable        .map_or_ok(|value| inspect_consumable(        value, &former_state, &updated_state))?;
	packet.equipment         .map_or_ok(|value| inspect_equipment(         value, &former_state, &updated_state))?;
	packet.name              .map_or_ok(|value| inspect_name(              value, &former_state, &updated_state))?;
	packet.skill_tree        .map_or_ok(|value| inspect_skill_tree(        value, &former_state, &updated_state))?;
	packet.mana_cubes        .map_or_ok(|value| inspect_mana_cubes(        value, &former_state, &updated_state))?;

	Ok(())
}

type Result = result::Result<(), String>;

trait Ensure {
	fn ensure<'a>(
		&self,
		property_name: &'a str,
		actual_value: &impl Debug,
		words: &'a str, //todo: come up with a better name (or drop this parameter entirely)
		allowed: &(impl Debug + ?Sized)
	) -> Result;
}
impl Ensure for bool {
	fn ensure<'a>(
		&self,
		property_name: &'a str,
		actual: &impl Debug,
		words: &'a str,
		allowed: &(impl Debug + ?Sized)
	) -> Result {
		self.ok_or(
			format!(
				"{} was {:?}, allowed was {} {:?}",
				property_name,
				actual,
				words,
				allowed
			)
		)
	}
}

trait EnsureNotNegative {
	fn ensure_not_negative(&self, property_name: &str) -> Result;
}
impl EnsureNotNegative for i32 {
	fn ensure_not_negative(&self, property_name: &str) -> Result {
		(!self.is_negative())//double negation in order for 0 to be Ok(())
			.ensure(property_name, self, "positive or", &0)
	}
}



trait EnsureAtMost: PartialOrd + Debug + Sized {
	fn ensure_at_most(&self, limit: Self, property_name: &str) -> Result {
		(*self <= limit)
			.ensure(property_name, self, "at most", &limit)
	}
}
impl<T: PartialOrd + Debug> EnsureAtMost for T {}



trait EnsureWithin: PartialOrd + Debug + Sized {
	fn ensure_within(&self, allowed_range: &(impl RangeBounds<Self> + Debug), property_name: &str) -> Result {
		allowed_range.contains(&self)
			.ensure(property_name, self, "within", &allowed_range)
	}
}
impl<T: PartialOrd + Debug> EnsureWithin for T {}



trait EnsureOneOf: PartialEq + Debug + Sized {
	fn ensure_one_of(&self, allowed_values: &[Self], property_name: &str) -> Result {
		allowed_values.contains(self)
			.ensure(property_name, self, "any of", &allowed_values)
	}

	fn ensure_exact(&self, allowed_value: &Self, property_name: &str) -> Result {
		(self == allowed_value)
			.ensure(property_name, self, "", allowed_value)
	}
}
impl<T: PartialEq + Debug> EnsureOneOf for T {}



trait PresentIn: PartialEq + Sized {
	fn present_in(&self, container: &[Self]) -> bool {
		container.contains(self)
	}
}

impl<T: PartialEq> PresentIn for T {}



trait MapOrOk<Value> {
	fn map_or_ok<Error>(&self, f: impl FnOnce(&Value) -> result::Result<(), Error>) -> result::Result<(), Error>;
}

impl<Value> MapOrOk<Value> for Option<Value> {
	fn map_or_ok<Error>(&self, f: impl FnOnce(&Value) -> result::Result<(), Error>) -> result::Result<(), Error> {
		match self {
			Some(x) => f(x),
			Option::None => Ok(())
		}
	}
}