use std::fmt::Debug;
use std::ops::RangeBounds;
use std::result;
use std::time::Instant;

use boolinator::Boolinator;
use tap::Tap;

use protocol::packet::CreatureUpdate;

use crate::server::player::Player;

use self::creature_update::*;

pub mod creature_update;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct PlayerData {
	combo_epoch: Option<Instant>,
	last_lag_spike: Option<Instant>,
	last_checked: Option<Instant>,
	total_shift_nanos: i64
}

#[expect(clippy::significant_drop_tightening, reason = "cannot drop any earlier")]
#[expect(clippy::cognitive_complexity, reason = "todo")]
pub async fn inspect_creature_update(source: &Player, packet: &CreatureUpdate) -> Result {
	let previous_state = source.character.read().await;
	let updated_state = previous_state.clone().tap_mut(|state| state.update(packet));

	packet.id.ensure_exact(&source.id, "creature_id")?;

	//todo: macro
	if packet.position         .is_some() { inspect_position         (&previous_state, &updated_state)? }
	if packet.rotation         .is_some() { inspect_rotation         (&previous_state, &updated_state)? }
	if packet.velocity         .is_some() { inspect_velocity         (&previous_state, &updated_state)? }
	if packet.acceleration     .is_some() { inspect_acceleration     (&previous_state, &updated_state)? }
	if packet.velocity_extra   .is_some() { inspect_velocity_extra   (&previous_state, &updated_state)? }
	if packet.head_tilt        .is_some() { inspect_head_tilt        (&previous_state, &updated_state)? }
	if packet.flags_physics    .is_some() { inspect_flags_physics    (&previous_state, &updated_state)? }
	if packet.affiliation      .is_some() { inspect_affiliation      (&previous_state, &updated_state)? }
	if packet.race             .is_some() { inspect_race             (&previous_state, &updated_state)? }
	if packet.animation        .is_some() { inspect_animation        (&previous_state, &updated_state)? }
	if packet.animation_time   .is_some() { inspect_animation_time   (&previous_state, &updated_state)? }
	if packet.combo            .is_some() { inspect_combo            (&previous_state, &updated_state)? }
	if packet.combo_timeout    .is_some() { inspect_combo_timeout    (&previous_state, &updated_state, source).await? } //todo: consistency
	if packet.appearance       .is_some() { inspect_appearance       (&previous_state, &updated_state)? }
	if packet.flags            .is_some() { inspect_flags            (&previous_state, &updated_state)? }
	if packet.effect_time_dodge.is_some() { inspect_effect_time_dodge(&previous_state, &updated_state)? }
	if packet.effect_time_stun .is_some() { inspect_effect_time_stun (&previous_state, &updated_state)? }
	if packet.effect_time_fear .is_some() { inspect_effect_time_fear (&previous_state, &updated_state)? }
	if packet.effect_time_chill.is_some() { inspect_effect_time_chill(&previous_state, &updated_state)? }
	if packet.effect_time_wind .is_some() { inspect_effect_time_wind (&previous_state, &updated_state)? }
	if packet.show_patch_time  .is_some() { inspect_show_patch_time  (&previous_state, &updated_state)? }
	if packet.occupation       .is_some() { inspect_occupation       (&previous_state, &updated_state)? }
	if packet.specialization   .is_some() { inspect_specialization   (&previous_state, &updated_state)? }
	if packet.mana_charge      .is_some() { inspect_mana_charge      (&previous_state, &updated_state)? }
	if packet.unknown24        .is_some() { inspect_unknown24        (&previous_state, &updated_state)? }
	if packet.unknown25        .is_some() { inspect_unknown25        (&previous_state, &updated_state)? }
	if packet.aim_offset       .is_some() { inspect_aim_offset       (&previous_state, &updated_state)? }
	if packet.health           .is_some() { inspect_health           (&previous_state, &updated_state)? }
	if packet.mana             .is_some() { inspect_mana             (&previous_state, &updated_state)? }
	if packet.blocking_gauge   .is_some() { inspect_blocking_gauge   (&previous_state, &updated_state)? }
	if packet.multipliers      .is_some() { inspect_multipliers      (&previous_state, &updated_state)? }
	if packet.unknown31        .is_some() { inspect_unknown31        (&previous_state, &updated_state)? }
	if packet.unknown32        .is_some() { inspect_unknown32        (&previous_state, &updated_state)? }
	if packet.level            .is_some() { inspect_level            (&previous_state, &updated_state)? }
	if packet.experience       .is_some() { inspect_experience       (&previous_state, &updated_state)? }
	if packet.master           .is_some() { inspect_master           (&previous_state, &updated_state)? }
	if packet.unknown36        .is_some() { inspect_unknown36        (&previous_state, &updated_state)? }
	if packet.rarity           .is_some() { inspect_rarity           (&previous_state, &updated_state)? }
	if packet.unknown38        .is_some() { inspect_unknown38        (&previous_state, &updated_state)? }
	if packet.home_zone        .is_some() { inspect_home_zone        (&previous_state, &updated_state)? }
	if packet.home             .is_some() { inspect_home             (&previous_state, &updated_state)? }
	if packet.zone_to_reveal   .is_some() { inspect_zone_to_reveal   (&previous_state, &updated_state)? }
	if packet.unknown42        .is_some() { inspect_unknown42        (&previous_state, &updated_state)? }
	if packet.consumable       .is_some() { inspect_consumable       (&previous_state, &updated_state)? }
	if packet.equipment        .is_some() { inspect_equipment        (&previous_state, &updated_state)? }
	if packet.name             .is_some() { inspect_name             (&previous_state, &updated_state)? }
	if packet.skill_tree       .is_some() { inspect_skill_tree       (&previous_state, &updated_state)? }
	if packet.mana_cubes       .is_some() { inspect_mana_cubes       (&previous_state, &updated_state)? }

	Ok(())
}

type Result = result::Result<(), String>;

trait Ensure {
	fn ensure(
		&self,
		property_name: &str,
		actual_value: &impl Debug,
		words: &str, //todo: come up with a better name (or drop this parameter entirely)
		allowed: &(impl Debug + ?Sized)
	) -> Result;
}
impl Ensure for bool {
	fn ensure(
		&self,
		property_name: &str,
		actual: &impl Debug,
		words: &str,
		allowed: &(impl Debug + ?Sized)
	) -> Result {
		self.ok_or(format!("{property_name} was {actual:?}, allowed was {words} {allowed:?}"))
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
		allowed_range.contains(self)
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