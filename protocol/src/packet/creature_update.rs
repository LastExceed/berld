use std::ffi::CStr;
use std::io::{Error, ErrorKind};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use nalgebra::Point3;
use rgb::RGB;

use crate::packet::*;
use crate::packet::common::EulerAngles;

impl CwSerializable for CreatureUpdate {
	fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
		//todo: can't decode from network stream directly because ???
		let size = reader.read_struct::<i32>()?;
		let mut buffer = vec![0u8; size as usize];
		reader.read_exact(&mut buffer)?;

		let mut decoder = ZlibDecoder::new(buffer.as_slice());

		let id = decoder.read_struct::<CreatureId>()?;
		let bitfield = decoder.read_struct::<u64>()?;

		//todo: macro
		let instance = Self {
			id,
			position             : if bitfield & (1 <<  0) > 0 { Some(decoder.read_struct()?) } else { None },
			rotation             : if bitfield & (1 <<  1) > 0 { Some(decoder.read_struct()?) } else { None },
			velocity             : if bitfield & (1 <<  2) > 0 { Some(decoder.read_struct()?) } else { None },
			acceleration         : if bitfield & (1 <<  3) > 0 { Some(decoder.read_struct()?) } else { None },
			velocity_extra       : if bitfield & (1 <<  4) > 0 { Some(decoder.read_struct()?) } else { None },
			climb_animation_state: if bitfield & (1 <<  5) > 0 { Some(decoder.read_struct()?) } else { None },
			flags_physics        : if bitfield & (1 <<  6) > 0 { Some(decoder.read_struct()?) } else { None },
			affiliation          : if bitfield & (1 <<  7) > 0 { Some(decoder.read_struct()?) } else { None },
			race                 : if bitfield & (1 <<  8) > 0 { Some(decoder.read_struct()?) } else { None },
			animation            : if bitfield & (1 <<  9) > 0 { Some(decoder.read_struct()?) } else { None },
			animation_time       : if bitfield & (1 << 10) > 0 { Some(decoder.read_struct()?) } else { None },
			combo                : if bitfield & (1 << 11) > 0 { Some(decoder.read_struct()?) } else { None },
			hit_time_out         : if bitfield & (1 << 12) > 0 { Some(decoder.read_struct()?) } else { None },
			appearance           : if bitfield & (1 << 13) > 0 { Some(decoder.read_struct()?) } else { None },
			flags                : if bitfield & (1 << 14) > 0 { Some(decoder.read_struct()?) } else { None },
			effect_time_dodge    : if bitfield & (1 << 15) > 0 { Some(decoder.read_struct()?) } else { None },
			effect_time_stun     : if bitfield & (1 << 16) > 0 { Some(decoder.read_struct()?) } else { None },
			effect_time_fear     : if bitfield & (1 << 17) > 0 { Some(decoder.read_struct()?) } else { None },
			effect_time_chill    : if bitfield & (1 << 18) > 0 { Some(decoder.read_struct()?) } else { None },
			effect_time_wind     : if bitfield & (1 << 19) > 0 { Some(decoder.read_struct()?) } else { None },
			show_patch_time      : if bitfield & (1 << 20) > 0 { Some(decoder.read_struct()?) } else { None },
			combat_class_major   : if bitfield & (1 << 21) > 0 { Some(decoder.read_struct()?) } else { None },
			combat_class_minor   : if bitfield & (1 << 22) > 0 { Some(decoder.read_struct()?) } else { None },
			mana_charge          : if bitfield & (1 << 23) > 0 { Some(decoder.read_struct()?) } else { None },
			unknown24            : if bitfield & (1 << 24) > 0 { Some(decoder.read_struct()?) } else { None },
			unknown25            : if bitfield & (1 << 25) > 0 { Some(decoder.read_struct()?) } else { None },
			aim_offset           : if bitfield & (1 << 26) > 0 { Some(decoder.read_struct()?) } else { None },
			health               : if bitfield & (1 << 27) > 0 { Some(decoder.read_struct()?) } else { None },
			mana                 : if bitfield & (1 << 28) > 0 { Some(decoder.read_struct()?) } else { None },
			blocking_gauge       : if bitfield & (1 << 29) > 0 { Some(decoder.read_struct()?) } else { None },
			multipliers          : if bitfield & (1 << 30) > 0 { Some(decoder.read_struct()?) } else { None },
			unknown31            : if bitfield & (1 << 31) > 0 { Some(decoder.read_struct()?) } else { None },
			unknown32            : if bitfield & (1 << 32) > 0 { Some(decoder.read_struct()?) } else { None },
			level                : if bitfield & (1 << 33) > 0 { Some(decoder.read_struct()?) } else { None },
			experience           : if bitfield & (1 << 34) > 0 { Some(decoder.read_struct()?) } else { None },
			master               : if bitfield & (1 << 35) > 0 { Some(decoder.read_struct()?) } else { None },
			unknown36            : if bitfield & (1 << 36) > 0 { Some(decoder.read_struct()?) } else { None },
			power_base           : if bitfield & (1 << 37) > 0 { Some(decoder.read_struct()?) } else { None },
			unknown38            : if bitfield & (1 << 38) > 0 { Some(decoder.read_struct()?) } else { None },
			home_zone            : if bitfield & (1 << 39) > 0 { Some(decoder.read_struct()?) } else { None },
			home                 : if bitfield & (1 << 40) > 0 { Some(decoder.read_struct()?) } else { None },
			zone_to_reveal       : if bitfield & (1 << 41) > 0 { Some(decoder.read_struct()?) } else { None },
			unknown42            : if bitfield & (1 << 42) > 0 { Some(decoder.read_struct()?) } else { None },
			consumable           : if bitfield & (1 << 43) > 0 { Some(decoder.read_struct()?) } else { None },
			equipment            : if bitfield & (1 << 44) > 0 { Some(decoder.read_struct()?) } else { None },
			name                 : if bitfield & (1 << 45) > 0 {
				if let Ok(cstr) = CStr::from_bytes_until_nul(decoder.read_struct::<[u8; 16]>()?.as_slice()) {
					Some(cstr.to_str().unwrap().to_string())
				} else {
					return Err(Error::from(ErrorKind::InvalidData));
				}
			} else { None },
			skill_tree           : if bitfield & (1 << 46) > 0 { Some(decoder.read_struct()?) } else { None },
			mana_cubes           : if bitfield & (1 << 47) > 0 { Some(decoder.read_struct()?) } else { None }
		};

		if !matches!(decoder.read_to_end(&mut vec![0u8; 0]), Ok(0)) {
			return Err(Error::from(ErrorKind::InvalidData));
		}
		Ok(instance)
	}

	fn write_to(&self, writer: &mut impl Write) -> Result<(), Error> {
		let mut bitfield = 0u64;

		//todo: macro
		bitfield |= (self.position             .is_some() as u64) <<  0;
		bitfield |= (self.rotation             .is_some() as u64) <<  1;
		bitfield |= (self.velocity             .is_some() as u64) <<  2;
		bitfield |= (self.acceleration         .is_some() as u64) <<  3;
		bitfield |= (self.velocity_extra       .is_some() as u64) <<  4;
		bitfield |= (self.climb_animation_state.is_some() as u64) <<  5;
		bitfield |= (self.flags_physics        .is_some() as u64) <<  6;
		bitfield |= (self.affiliation          .is_some() as u64) <<  7;
		bitfield |= (self.race                 .is_some() as u64) <<  8;
		bitfield |= (self.animation            .is_some() as u64) <<  9;
		bitfield |= (self.animation_time       .is_some() as u64) << 10;
		bitfield |= (self.combo                .is_some() as u64) << 11;
		bitfield |= (self.hit_time_out         .is_some() as u64) << 12;
		bitfield |= (self.appearance           .is_some() as u64) << 13;
		bitfield |= (self.flags                .is_some() as u64) << 14;
		bitfield |= (self.effect_time_dodge    .is_some() as u64) << 15;
		bitfield |= (self.effect_time_stun     .is_some() as u64) << 16;
		bitfield |= (self.effect_time_fear     .is_some() as u64) << 17;
		bitfield |= (self.effect_time_chill    .is_some() as u64) << 18;
		bitfield |= (self.effect_time_wind     .is_some() as u64) << 19;
		bitfield |= (self.show_patch_time      .is_some() as u64) << 20;
		bitfield |= (self.combat_class_major   .is_some() as u64) << 21;
		bitfield |= (self.combat_class_minor   .is_some() as u64) << 22;
		bitfield |= (self.mana_charge          .is_some() as u64) << 23;
		bitfield |= (self.unknown24            .is_some() as u64) << 24;
		bitfield |= (self.unknown25            .is_some() as u64) << 25;
		bitfield |= (self.aim_offset           .is_some() as u64) << 26;
		bitfield |= (self.health               .is_some() as u64) << 27;
		bitfield |= (self.mana                 .is_some() as u64) << 28;
		bitfield |= (self.blocking_gauge       .is_some() as u64) << 29;
		bitfield |= (self.multipliers          .is_some() as u64) << 30;
		bitfield |= (self.unknown31            .is_some() as u64) << 31;
		bitfield |= (self.unknown32            .is_some() as u64) << 32;
		bitfield |= (self.level                .is_some() as u64) << 33;
		bitfield |= (self.experience           .is_some() as u64) << 34;
		bitfield |= (self.master               .is_some() as u64) << 35;
		bitfield |= (self.unknown36            .is_some() as u64) << 36;
		bitfield |= (self.power_base           .is_some() as u64) << 37;
		bitfield |= (self.unknown38            .is_some() as u64) << 38;
		bitfield |= (self.home_zone            .is_some() as u64) << 39;
		bitfield |= (self.home                 .is_some() as u64) << 40;
		bitfield |= (self.zone_to_reveal       .is_some() as u64) << 41;
		bitfield |= (self.unknown42            .is_some() as u64) << 42;
		bitfield |= (self.consumable           .is_some() as u64) << 43;
		bitfield |= (self.equipment            .is_some() as u64) << 44;
		bitfield |= (self.name                 .is_some() as u64) << 45;
		bitfield |= (self.skill_tree           .is_some() as u64) << 46;
		bitfield |= (self.mana_cubes           .is_some() as u64) << 47;

		let mut buffer = vec![];
		{
			let mut encoder = ZlibEncoder::new(&mut buffer, Compression::default());

			encoder.write_struct(&self.id)?;
			encoder.write_struct(&bitfield)?;

			//todo: macro
			if let Some(it) = &self.position              { encoder.write_struct(it)?; }
			if let Some(it) = &self.rotation              { encoder.write_struct(it)?; }
			if let Some(it) = &self.velocity              { encoder.write_struct(it)?; }
			if let Some(it) = &self.acceleration          { encoder.write_struct(it)?; }
			if let Some(it) = &self.velocity_extra        { encoder.write_struct(it)?; }
			if let Some(it) = &self.climb_animation_state { encoder.write_struct(it)?; }
			if let Some(it) = &self.flags_physics         { encoder.write_struct(it)?; }
			if let Some(it) = &self.affiliation           { encoder.write_struct(it)?; }
			if let Some(it) = &self.race                  { encoder.write_struct(it)?; }
			if let Some(it) = &self.animation             { encoder.write_struct(it)?; }
			if let Some(it) = &self.animation_time        { encoder.write_struct(it)?; }
			if let Some(it) = &self.combo                 { encoder.write_struct(it)?; }
			if let Some(it) = &self.hit_time_out          { encoder.write_struct(it)?; }
			if let Some(it) = &self.appearance            { encoder.write_struct(it)?; }
			if let Some(it) = &self.flags                 { encoder.write_struct(it)?; }
			if let Some(it) = &self.effect_time_dodge     { encoder.write_struct(it)?; }
			if let Some(it) = &self.effect_time_stun      { encoder.write_struct(it)?; }
			if let Some(it) = &self.effect_time_fear      { encoder.write_struct(it)?; }
			if let Some(it) = &self.effect_time_chill     { encoder.write_struct(it)?; }
			if let Some(it) = &self.effect_time_wind      { encoder.write_struct(it)?; }
			if let Some(it) = &self.show_patch_time       { encoder.write_struct(it)?; }
			if let Some(it) = &self.combat_class_major    { encoder.write_struct(it)?; }
			if let Some(it) = &self.combat_class_minor    { encoder.write_struct(it)?; }
			if let Some(it) = &self.mana_charge           { encoder.write_struct(it)?; }
			if let Some(it) = &self.unknown24             { encoder.write_struct(it)?; }
			if let Some(it) = &self.unknown25             { encoder.write_struct(it)?; }
			if let Some(it) = &self.aim_offset            { encoder.write_struct(it)?; }
			if let Some(it) = &self.health                { encoder.write_struct(it)?; }
			if let Some(it) = &self.mana                  { encoder.write_struct(it)?; }
			if let Some(it) = &self.blocking_gauge        { encoder.write_struct(it)?; }
			if let Some(it) = &self.multipliers           { encoder.write_struct(it)?; }
			if let Some(it) = &self.unknown31             { encoder.write_struct(it)?; }
			if let Some(it) = &self.unknown32             { encoder.write_struct(it)?; }
			if let Some(it) = &self.level                 { encoder.write_struct(it)?; }
			if let Some(it) = &self.experience            { encoder.write_struct(it)?; }
			if let Some(it) = &self.master                { encoder.write_struct(it)?; }
			if let Some(it) = &self.unknown36             { encoder.write_struct(it)?; }
			if let Some(it) = &self.power_base            { encoder.write_struct(it)?; }
			if let Some(it) = &self.unknown38             { encoder.write_struct(it)?; }
			if let Some(it) = &self.home_zone             { encoder.write_struct(it)?; }
			if let Some(it) = &self.home                  { encoder.write_struct(it)?; }
			if let Some(it) = &self.zone_to_reveal        { encoder.write_struct(it)?; }
			if let Some(it) = &self.unknown42             { encoder.write_struct(it)?; }
			if let Some(it) = &self.consumable            { encoder.write_struct(it)?; }
			if let Some(it) = &self.equipment             { encoder.write_struct(it)?; }
			if let Some(it) = &self.name                  {
				let bytes = it.as_bytes();
				if bytes.len() > 16 { return Err(Error::from(ErrorKind::InvalidData)) }
				encoder.write_all(bytes)?;
				encoder.write_all(&vec![0u8; 16 - bytes.len()])?;
				//todo: check what happens with non-ascii characters
			}
			if let Some(it) = &self.skill_tree            { encoder.write_struct(it)?; }
			if let Some(it) = &self.mana_cubes            { encoder.write_struct(it)?; }

			encoder.flush()?;
		}

		writer.write_struct(&(buffer.len() as i32))?;
		writer.write_all(&buffer)
	}
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum PhysicsFlag {
	OnGround,
	Swimming,
	TouchingWall,

	//#4 always true
	PushingWall = 5,
	PushingObject
}
impl From<PhysicsFlag> for u32 {
	fn from(it: PhysicsFlag) -> Self {
		it as Self
	}
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Affiliation {
	Player,
	Enemy,

	NPC = 3,

	Pet = 5,
	Neutral
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum Animation {
	Idle,
	DualWieldM1a,
	DualWieldM1b,
	Unknown003, //like daggers
	Unknown004,
	LongswordM2,
	UnarmedM1a, //fists use these
	UnarmedM1b,
	ShieldM2Charging,
	ShieldM1a,
	ShieldM1b,
	UnarmedM2,
	Unknown012, //swords rip apart
	LongswordM1a,
	LongswordM1b,
	Unknown015, //probably for greatweapon A1
	Unknown016, //same as 17
	DaggersM2,
	DaggersM1a,
	DaggersM1b,
	FistsM2,
	Kick,
	ShootArrow,
	CrossbowM2,
	CrossbowM2Charging,
	BowM2Charging,
	BoomerangM1,
	BoomerangM2Charging,
	BeamDraining,
	Unknown029, //nothing
	StaffFireM1,
	StaffFireM2,
	StaffWaterM1,
	StaffWaterM2,
	HealingStream,
	Unknown035, //summon animation
	Unknown036, //wand charging?
	BraceletFireM2,
	WandFireM1,
	BraceletsFireM1a,
	BraceletsFireM1b,
	BraceletsWaterM1a,
	BraceletsWaterM1b,
	BraceletWaterM2,
	WandWaterM1,
	WandWaterM2,
	WandFireM2,
	Unknown047, //same as smash
	Intercept,
	Teleport,
	Unknown050,
	Unknown051, //mob attack?
	Unknown052, //nothing, immediately switches to 0
	Unknown053, //nothing
	Smash,
	BowM2,
	Unknown056, //nothing, causes rotation lock
	GreatweaponM1a,
	GreatweaponM1c,
	GreatweaponM2Charging,
	GreatweaponM2Berserker,
	GreatweaponM2Guardian,
	Unknown062, //probably for greatweapon A2
	UnarmedM2Charging,
	DualWieldM2Charging,
	Unknown065, //probably for greatweapon B1
	Unknown066, //probably for greatweapon B2
	GreatweaponM1b,
	BossCharge1,
	BossCharge2,
	BossSpinkick,
	BossBlock,
	BossSpin,
	BossCry,
	BossStomp,
	BossKick,
	BossKnockdownForward,
	BossKnockdownLeft,
	BossKnockdownRight,
	Stealth,
	Drinking,
	Eating,
	PetFoodPresent,
	Sitting,
	Sleeping,
	Unknown085, //nothing
	Cyclone,
	FireExplosionLong,
	FireExplosioni16,
	Lava,
	Splash,
	EarthQuake,
	Clone,
	Unknown093, //same as intercept
	FireBeam,
	FireRay,
	Shuriken,
	Unknown097, //nothing, rotation lock
	Unknown098, //parry? causes blocking
	Unknown099, //nothing, rotation lock
	Unknown100, //nothing
	SuperBulwalk, //casues bulwalk
	Unknown102, //nothing
	SuperManaShield, //causes manashield
	ShieldM2,
	TeleportToCity,
	Riding,
	Boat,
	Boulder,
	ManaCubePickup,
	Unknown110 //mob attack?
}

#[repr(C)]
#[derive(Clone)]
pub struct Appearance {
	pub unknown: i16,
	pub hair_color: RGB<u8>,
	//pad1
	pub flags: FlagSet16<AppearanceFlag>,
	pub creature_size: Hitbox,
	pub head_model: i16,
	pub hair_model: i16,
	pub hand_model: i16,
	pub foot_model: i16,
	pub body_model: i16,
	pub tail_model: i16,
	pub shoulder2model: i16,
	pub wing_model: i16,
	pub head_size: f32,
	pub body_size: f32,
	pub hand_size: f32,
	pub foot_size: f32,
	pub shoulder2size: f32,
	pub weapon_size: f32,
	pub tail_size: f32,
	pub shoulder1size: f32,
	pub wing_size: f32,
	pub body_rotation: f32,
	pub hand_rotation: EulerAngles,
	pub feet_rotation: f32,
	pub wing_rotation: f32,
	pub tail_rotation: f32,
	pub body_offset: Point3<f32>,
	pub head_offset: Point3<f32>,
	pub hand_offset: Point3<f32>,
	pub foot_offset: Point3<f32>,
	pub tail_offset: Point3<f32>,
	pub wing_offset: Point3<f32>
}

#[repr(u16)]
#[derive(Copy, Clone)]
pub enum AppearanceFlag {
	FourLegged,
	CanFly,






	Immovable = 8, //found on dummies
	BossGlow,


	//#12 found on bosses
	Invincible = 13, //found on dummies
}
impl From<AppearanceFlag> for u16 {
	fn from(it: AppearanceFlag) -> Self {
		it as Self
	}
}

#[repr(u16)]
#[derive(Copy, Clone)]
pub enum CreatureFlag {
	Climbing,

	Aiming = 2,

	Gliding = 4,
	FriendlyFire,
	Sprinting,


	Lamp = 9,
	Sniping,
}
impl From<CreatureFlag> for u16 {
	fn from(it: CreatureFlag) -> Self {
		it as Self
	}
}

#[repr(i8)]
#[derive(Copy, Clone)]
pub enum CombatClassMajor {
	None,
	Warrior,
	Ranger,
	Mage,
	Rogue,

	GeneralShopkeep = -128,
	WeaponShopkeep,
	ArmorShopkeep,
	Identifier,
	Innkeep,
	Blacksmith,//no function
	Woodworker,//no function
	Weaver,//no function
	Villager,
	Adapter
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum CombatClassMinor {
	Default,
	Alternative,
	Witch
}

#[repr(C)]
#[derive(Clone)]
pub struct Multipliers {
	pub health: f32,
	pub attack_speed: f32,
	pub damage: f32,
	pub armor: f32,
	pub resi: f32
}

#[repr(C)]
#[derive(Clone)]
pub struct Equipment {
	pub unknown: Item,
	pub neck: Item,
	pub chest: Item,
	pub feet: Item,
	pub hands: Item,
	pub shoulder: Item,
	pub left_weapon: Item,
	pub right_weapon: Item,
	pub left_ring: Item,
	pub right_ring: Item,
	pub lamp: Item,
	pub special: Item,
	pub pet: Item
}

#[repr(C)]
#[derive(Clone)]
pub struct SkillTree {
	pub pet_master: i32,
	pub pet_riding: i32,
	pub sailing: i32,
	pub climbing: i32,
	pub hang_gliding: i32,
	pub swimming: i32,
	pub ability1: i32,
	pub ability2: i32,
	pub ability3: i32,
	pub ability4: i32,
	pub ability5: i32
}