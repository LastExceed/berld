use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::RangeBounds;
use std::result;
use std::time::Instant;

use boolinator::Boolinator;
use tokio::sync::RwLock;

use protocol::packet::common::CreatureId;
use protocol::packet::CreatureUpdate;

use crate::server::creature::Creature;
use crate::server::player::Player;

use self::creature_update::*;

pub mod creature_update;

struct PlayerACData {
	last_combo_update: Option<Instant>
}

pub struct AntiCheat {
	ac_datas: RwLock<HashMap<CreatureId, PlayerACData>>
}

impl AntiCheat {
	pub fn new() -> Self {
		Self {
			ac_datas: RwLock::new(HashMap::new())
		}
	}

	pub async fn on_join(&self, player: &Player) {
		self.ac_datas.write().await
			.insert(player.id, PlayerACData { last_combo_update: None });
	}

	pub async fn on_leave(&self, player: &Player) {
		self.ac_datas.write().await
			.remove(&player.id);
	}


	pub async fn inspect_creature_update(&self, source: &Player, packet: &CreatureUpdate, former_state: &Creature, updated_state: &Creature) -> Result {
		packet.id.ensure_exact(&source.id, "creature_id")?;

		let mut map_guard = self.ac_datas.write().await;
		let Some(ac_data) = map_guard.get_mut(&packet.id)
			else { unreachable!() };//should be unreachable
		if former_state.health == 0.0 && updated_state.health > 0.0 {
			ac_data.last_combo_update = Some(Instant::now()); //clocks freeze on death, so there's a desync on respawn
		}

		//todo: macro
		if let Some(ref value) = packet.position          { inspect_position         (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.rotation          { inspect_rotation         (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.velocity          { inspect_velocity         (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.acceleration      { inspect_acceleration     (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.velocity_extra    { inspect_velocity_extra   (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.head_tilt         { inspect_head_tilt        (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.flags_physics     { inspect_flags_physics    (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.affiliation       { inspect_affiliation      (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.race              { inspect_race             (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.animation         { inspect_animation        (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.animation_time    { inspect_animation_time   (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.combo             { inspect_combo            (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.combo_timeout     { inspect_combo_timeout    (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.appearance        { inspect_appearance       (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.flags             { inspect_flags            (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.effect_time_dodge { inspect_effect_time_dodge(value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.effect_time_stun  { inspect_effect_time_stun (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.effect_time_fear  { inspect_effect_time_fear (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.effect_time_chill { inspect_effect_time_chill(value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.effect_time_wind  { inspect_effect_time_wind (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.show_patch_time   { inspect_show_patch_time  (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.occupation        { inspect_occupation       (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.specialization    { inspect_specialization   (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.mana_charge       { inspect_mana_charge      (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.unknown24         { inspect_unknown24        (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.unknown25         { inspect_unknown25        (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.aim_offset        { inspect_aim_offset       (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.health            { inspect_health           (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.mana              { inspect_mana             (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.blocking_gauge    { inspect_blocking_gauge   (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.multipliers       { inspect_multipliers      (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.unknown31         { inspect_unknown31        (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.unknown32         { inspect_unknown32        (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.level             { inspect_level            (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.experience        { inspect_experience       (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.master            { inspect_master           (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.unknown36         { inspect_unknown36        (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.rarity            { inspect_rarity           (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.unknown38         { inspect_unknown38        (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.home_zone         { inspect_home_zone        (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.home              { inspect_home             (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.zone_to_reveal    { inspect_zone_to_reveal   (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.unknown42         { inspect_unknown42        (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.consumable        { inspect_consumable       (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.equipment         { inspect_equipment        (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.name              { inspect_name             (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.skill_tree        { inspect_skill_tree       (value, former_state, updated_state, ac_data)? };
		if let Some(ref value) = packet.mana_cubes        { inspect_mana_cubes       (value, former_state, updated_state, ac_data)? };

		Ok(())
	}
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