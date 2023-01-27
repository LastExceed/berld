use protocol::nalgebra::{Point3, Vector3};
use protocol::packet::common::{CreatureId, EulerAngles, Item, Race};
use protocol::packet::common::item::Stat;
use protocol::packet::creature_update::*;
use protocol::packet::creature_update::multipliers::Multiplier::Health;
use protocol::packet::creature_update::Occupation::*;
use protocol::packet::creature_update::Specialization::Alternative;
use protocol::packet::CreatureUpdate;
use protocol::utils::{level_scaling_factor, rarity_scaling_factor};
use protocol::utils::constants::CombatClass;
use protocol::utils::flagset::{FlagSet16, FlagSet32};

#[derive(Clone)]
pub struct Creature {
	pub position: Point3<i64>,
	pub rotation: EulerAngles,
	pub velocity: Vector3<f32>,
	pub acceleration: Vector3<f32>,
	/**used by the 'retreat' ability*/
	pub velocity_extra: Vector3<f32>,
	pub head_tilt: f32,
	pub flags_physics: FlagSet32<PhysicsFlag>,
	pub affiliation: Affiliation,
	pub race: Race,
	pub animation: Animation,
	pub animation_time: i32,
	pub combo: i32,
	pub hit_time_out: i32,
	pub appearance: Appearance,
	pub flags: FlagSet16<CreatureFlag>,
	pub effect_time_dodge: i32,
	pub effect_time_stun: i32,
	pub effect_time_fear: i32,
	pub effect_time_chill: i32,
	pub effect_time_wind: i32,
	/**unknown purpose, name adopted from cuwo*/
	pub show_patch_time: i32,
	pub occupation: Occupation,
	pub specialization: Specialization,
	pub mana_charge: f32,
	pub unknown24: [f32; 3],
	pub unknown25: [f32; 3],
	/**coordinates of the location this creature is aiming at, relative to its own position*/
	pub aim_displacement: Point3<f32>,
	pub health: f32,
	pub mana: f32,
	pub blocking_gauge: f32,
	pub multipliers: Multipliers,
	pub unknown31: i8,
	pub unknown32: i8,
	pub level: i32,
	pub experience: i32,
	/**for pets this is the [CreatureId] of their owner*/
	pub master: CreatureId,
	pub unknown36: i64,
	/**this is the '+#' that monsters in some dungeons have next to their [race]*/
	pub power_base: u8,
	pub unknown38: i32,
	pub home_zone: Point3<i32>,
	pub home: Point3<i64>,
	/**players within ±2 [level] of the dungeon in this zone see a green speech bubble above this creature, and can get this zone revealed on the map by talking to this creature*/
	pub zone_to_reveal: Point3<i32>,
	pub unknown42: i8,//0 3 4 for villages - 3 = dialog about pet food
	pub consumable: Item,
	pub equipment: Equipment,
	pub name: String,
	pub skill_tree: SkillTree,
	pub mana_cubes: i32
}

impl Creature {
	pub fn combat_class(&self) -> CombatClass {
		CombatClass {
			occupation: self.occupation,
			specialization: self.specialization
		}
	}

	pub fn maybe_from(creature_update: &CreatureUpdate) -> Option<Creature> {
		//todo: macro?
		Some(Self {
			position             : creature_update.position?,
			rotation             : creature_update.rotation.clone()?,
			velocity             : creature_update.velocity?,
			acceleration         : creature_update.acceleration?,
			velocity_extra       : creature_update.velocity_extra?,
			head_tilt            : creature_update.head_tilt?,
			flags_physics        : creature_update.flags_physics.clone()?,
			affiliation          : creature_update.affiliation?,
			race                 : creature_update.race?,
			animation            : creature_update.animation?,
			animation_time       : creature_update.animation_time?,
			combo                : creature_update.combo?,
			hit_time_out         : creature_update.combo_timeout?,
			appearance           : creature_update.appearance.clone()?,
			flags                : creature_update.flags.clone()?,
			effect_time_dodge    : creature_update.effect_time_dodge?,
			effect_time_stun     : creature_update.effect_time_stun?,
			effect_time_fear     : creature_update.effect_time_fear?,
			effect_time_chill    : creature_update.effect_time_chill?,
			effect_time_wind     : creature_update.effect_time_wind?,
			show_patch_time      : creature_update.show_patch_time?,
			occupation           : creature_update.occupation?,
			specialization       : creature_update.specialization?,
			mana_charge          : creature_update.mana_charge?,
			unknown24            : creature_update.unknown24?,
			unknown25            : creature_update.unknown25?,
			aim_displacement     : creature_update.aim_offset?,
			health               : creature_update.health?,
			mana                 : creature_update.mana?,
			blocking_gauge       : creature_update.blocking_gauge?,
			multipliers          : creature_update.multipliers.clone()?,
			unknown31            : creature_update.unknown31?,
			unknown32            : creature_update.unknown32?,
			level                : creature_update.level?,
			experience           : creature_update.experience?,
			master               : creature_update.master?,
			unknown36            : creature_update.unknown36?,
			power_base           : creature_update.power_base?,
			unknown38            : creature_update.unknown38?,
			home_zone            : creature_update.home_zone?,
			home                 : creature_update.home?,
			zone_to_reveal       : creature_update.zone_to_reveal?,
			unknown42            : creature_update.unknown42?,
			consumable           : creature_update.consumable.clone()?,
			equipment            : creature_update.equipment.clone()?,
			name                 : creature_update.name.clone()?,
			skill_tree           : creature_update.skill_tree.clone()?,
			mana_cubes           : creature_update.mana_cubes?
		})
	}

	pub fn update(&mut self, packet: &CreatureUpdate) {
		//todo: macro
		if let Some(it) = packet.position              { self.position              = it }
		if let Some(it) = packet.rotation.clone()      { self.rotation              = it }
		if let Some(it) = packet.velocity              { self.velocity              = it }
		if let Some(it) = packet.acceleration          { self.acceleration          = it }
		if let Some(it) = packet.velocity_extra        { self.velocity_extra        = it }
		if let Some(it) = packet.head_tilt             { self.head_tilt             = it }
		if let Some(it) = packet.flags_physics.clone() { self.flags_physics         = it }
		if let Some(it) = packet.affiliation           { self.affiliation           = it }
		if let Some(it) = packet.race                  { self.race                  = it }
		if let Some(it) = packet.animation             { self.animation             = it }
		if let Some(it) = packet.animation_time        { self.animation_time        = it }
		if let Some(it) = packet.combo                 { self.combo                 = it }
		if let Some(it) = packet.combo_timeout         { self.hit_time_out          = it }
		if let Some(it) = packet.appearance.clone()    { self.appearance            = it }
		if let Some(it) = packet.flags.clone()         { self.flags                 = it }
		if let Some(it) = packet.effect_time_dodge     { self.effect_time_dodge     = it }
		if let Some(it) = packet.effect_time_stun      { self.effect_time_stun      = it }
		if let Some(it) = packet.effect_time_fear      { self.effect_time_fear      = it }
		if let Some(it) = packet.effect_time_chill     { self.effect_time_chill     = it }
		if let Some(it) = packet.effect_time_wind      { self.effect_time_wind      = it }
		if let Some(it) = packet.show_patch_time       { self.show_patch_time       = it }
		if let Some(it) = packet.occupation            { self.occupation = it }
		if let Some(it) = packet.specialization        { self.specialization = it }
		if let Some(it) = packet.mana_charge           { self.mana_charge           = it }
		if let Some(it) = packet.unknown24             { self.unknown24             = it }
		if let Some(it) = packet.unknown25             { self.unknown25             = it }
		if let Some(it) = packet.aim_offset            { self.aim_displacement      = it }
		if let Some(it) = packet.health                { self.health                = it }
		if let Some(it) = packet.mana                  { self.mana                  = it }
		if let Some(it) = packet.blocking_gauge        { self.blocking_gauge        = it }
		if let Some(it) = packet.multipliers.clone()   { self.multipliers           = it }
		if let Some(it) = packet.unknown31             { self.unknown31             = it }
		if let Some(it) = packet.unknown32             { self.unknown32             = it }
		if let Some(it) = packet.level                 { self.level                 = it }
		if let Some(it) = packet.experience            { self.experience            = it }
		if let Some(it) = packet.master                { self.master                = it }
		if let Some(it) = packet.unknown36             { self.unknown36             = it }
		if let Some(it) = packet.power_base            { self.power_base            = it }
		if let Some(it) = packet.unknown38             { self.unknown38             = it }
		if let Some(it) = packet.home_zone             { self.home_zone             = it }
		if let Some(it) = packet.home                  { self.home                  = it }
		if let Some(it) = packet.zone_to_reveal        { self.zone_to_reveal        = it }
		if let Some(it) = packet.unknown42             { self.unknown42             = it }
		if let Some(it) = packet.consumable.clone()    { self.consumable            = it }
		if let Some(it) = packet.equipment.clone()     { self.equipment             = it }
		if let Some(it) = packet.name.clone()          { self.name                  = it }
		if let Some(it) = packet.skill_tree.clone()    { self.skill_tree            = it }
		if let Some(it) = packet.mana_cubes            { self.mana_cubes            = it }
	}

	pub fn to_update(&self, id: CreatureId) -> CreatureUpdate {
		CreatureUpdate {
			id,
			position          : Some(self.position),
			rotation          : Some(self.rotation.clone()),
			velocity          : Some(self.velocity),
			acceleration      : Some(self.acceleration),
			velocity_extra    : Some(self.velocity_extra),
			head_tilt         : Some(self.head_tilt),
			flags_physics     : Some(self.flags_physics.clone()),
			affiliation       : Some(self.affiliation),
			race              : Some(self.race),
			animation         : Some(self.animation),
			animation_time    : Some(self.animation_time),
			combo             : Some(self.combo),
			combo_timeout     : Some(self.hit_time_out),
			appearance        : Some(self.appearance.clone()),
			flags             : Some(self.flags.clone()),
			effect_time_dodge : Some(self.effect_time_dodge),
			effect_time_stun  : Some(self.effect_time_stun),
			effect_time_fear  : Some(self.effect_time_fear),
			effect_time_chill : Some(self.effect_time_chill),
			effect_time_wind  : Some(self.effect_time_wind),
			show_patch_time   : Some(self.show_patch_time),
			occupation        : Some(self.occupation),
			specialization    : Some(self.specialization),
			mana_charge       : Some(self.mana_charge),
			unknown24         : Some(self.unknown24),
			unknown25         : Some(self.unknown25),
			aim_offset        : Some(self.aim_displacement),
			health            : Some(self.health),
			mana              : Some(self.mana),
			blocking_gauge    : Some(self.blocking_gauge),
			multipliers       : Some(self.multipliers.clone()),
			unknown31         : Some(self.unknown31),
			unknown32         : Some(self.unknown32),
			level             : Some(self.level),
			experience        : Some(self.experience),
			master            : Some(self.master),
			unknown36         : Some(self.unknown36),
			power_base        : Some(self.power_base),
			unknown38         : Some(self.unknown38),
			home_zone         : Some(self.home_zone),
			home              : Some(self.home),
			zone_to_reveal    : Some(self.zone_to_reveal.clone()),
			unknown42         : Some(self.unknown42),
			consumable        : Some(self.consumable.clone()),
			equipment         : Some(self.equipment.clone()),
			name              : Some(self.name.clone()),
			skill_tree        : Some(self.skill_tree.clone()),
			mana_cubes        : Some(self.mana_cubes),
		}
	}

	pub fn maximum_health(&self) -> f32 {
		let combat_class_multiplier =
			match self.occupation {
				Warrior => 1.3 * if self.specialization == Alternative { 1.25 } else { 1.0 },
				Ranger  => 1.1,
				Rogue   => 1.2,
				_       => 1.0
			};

		let innate_health = [
			level_scaling_factor(self.level as f32),
			rarity_scaling_factor(if self.affiliation == Affiliation::Player { 4 } else { self.power_base }),
			combat_class_multiplier
		].iter().fold(
			self.multipliers[Health],
			|accumulator, multiplier| accumulator * multiplier
		);

		let equipment_bonus = self.equipment.iter().map(|item| item.stats()[Stat::Health]).sum::<f32>();

		innate_health + equipment_bonus
	}
}