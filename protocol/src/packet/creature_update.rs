use std::ffi::CStr;
use std::io::ErrorKind::InvalidData;

use async_compression::tokio::bufread::ZlibDecoder;
use async_compression::tokio::write::ZlibEncoder;
use nalgebra::Point3;
use rgb::RGB;
use strum_macros::EnumIter;
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{ReadCwData, Validate, Validator};
use crate::packet::*;
use crate::packet::common::EulerAngles;
use crate::packet::creature_update::equipment::Slot;
use crate::packet::creature_update::multipliers::Multiplier;
use crate::packet::creature_update::skill_tree::Skill;
use crate::utils::ArrayWrapper;

pub mod equipment;
pub mod skill_tree;
pub mod multipliers;

impl<Readable: AsyncRead + Unpin> ReadCwData<CreatureUpdate> for Readable {
	async fn read_cw_data(&mut self) -> io::Result<CreatureUpdate> {
		//todo: can't decode from network stream directly because ???
		let size = self.read_arbitrary::<i32>().await?;
		let mut buffer = vec![0u8; size as usize];
		self.read_exact(&mut buffer).await?;

		let mut decoder = ZlibDecoder::new(buffer.as_slice());

		let id = decoder.read_arbitrary::<CreatureId>().await?;
		let bitfield = decoder.read_arbitrary::<u64>().await?;

		//todo: macro
		let instance = CreatureUpdate {
			id,
			position          : if bitfield & (1 <<  0) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			rotation          : if bitfield & (1 <<  1) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			velocity          : if bitfield & (1 <<  2) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			acceleration      : if bitfield & (1 <<  3) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			velocity_extra    : if bitfield & (1 <<  4) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			head_tilt         : if bitfield & (1 <<  5) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			flags_physics     : if bitfield & (1 <<  6) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			affiliation       : if bitfield & (1 <<  7) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			race              : if bitfield & (1 <<  8) > 0 {
				let race = decoder.read_arbitrary().await?;
				//the game treats Race as u32 here, but u8 everywhere else
				//so we need to skip 3 bytes here
				let padding = decoder.read_arbitrary::<[u8;3]>().await?;
				if padding != [0u8; 3] {
					return Err(InvalidData.into());
				}
				Some(race)
			} else { None },
			animation         : if bitfield & (1 <<  9) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			animation_time    : if bitfield & (1 << 10) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			combo             : if bitfield & (1 << 11) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			combo_timeout     : if bitfield & (1 << 12) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			appearance        : if bitfield & (1 << 13) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			flags             : if bitfield & (1 << 14) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			effect_time_dodge : if bitfield & (1 << 15) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			effect_time_stun  : if bitfield & (1 << 16) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			effect_time_fear  : if bitfield & (1 << 17) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			effect_time_chill : if bitfield & (1 << 18) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			effect_time_wind  : if bitfield & (1 << 19) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			show_patch_time   : if bitfield & (1 << 20) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			occupation        : if bitfield & (1 << 21) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			specialization    : if bitfield & (1 << 22) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			mana_charge       : if bitfield & (1 << 23) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			unknown24         : if bitfield & (1 << 24) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			unknown25         : if bitfield & (1 << 25) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			aim_offset        : if bitfield & (1 << 26) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			health            : if bitfield & (1 << 27) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			mana              : if bitfield & (1 << 28) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			blocking_gauge    : if bitfield & (1 << 29) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			multipliers       : if bitfield & (1 << 30) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			unknown31         : if bitfield & (1 << 31) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			unknown32         : if bitfield & (1 << 32) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			level             : if bitfield & (1 << 33) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			experience        : if bitfield & (1 << 34) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			master            : if bitfield & (1 << 35) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			unknown36         : if bitfield & (1 << 36) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			rarity            : if bitfield & (1 << 37) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			unknown38         : if bitfield & (1 << 38) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			home_zone         : if bitfield & (1 << 39) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			home              : if bitfield & (1 << 40) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			zone_to_reveal    : if bitfield & (1 << 41) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			unknown42         : if bitfield & (1 << 42) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			consumable        : if bitfield & (1 << 43) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			equipment         : if bitfield & (1 << 44) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			name              : if bitfield & (1 << 45) > 0 {
				let name = CStr::from_bytes_until_nul(decoder.read_arbitrary::<[u8; 16]>().await?.as_slice())
					.map_err(|_| io::Error::from(InvalidData))?
					.to_str()
					.map_err(|_| io::Error::from(InvalidData))?
					.to_string();

				Some(name)
			} else { None },
			skill_tree        : if bitfield & (1 << 46) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			mana_cubes        : if bitfield & (1 << 47) > 0 { Some(decoder.read_arbitrary().await?) } else { None }
		};

		if !matches!(decoder.read_to_end(&mut vec![0u8; 0]).await, Ok(0)) {
			return Err(InvalidData.into());
		}
		Ok(instance)
	}
}

impl<Writable: AsyncWrite + Unpin> WriteCwData<CreatureUpdate> for Writable {
	async fn write_cw_data(&mut self, creature_update: &CreatureUpdate) -> std::io::Result<()> {
		let mut bitfield = 0u64;

		//todo: macro
		bitfield |= (creature_update.position         .is_some() as u64) <<  0;
		bitfield |= (creature_update.rotation         .is_some() as u64) <<  1;
		bitfield |= (creature_update.velocity         .is_some() as u64) <<  2;
		bitfield |= (creature_update.acceleration     .is_some() as u64) <<  3;
		bitfield |= (creature_update.velocity_extra   .is_some() as u64) <<  4;
		bitfield |= (creature_update.head_tilt        .is_some() as u64) <<  5;
		bitfield |= (creature_update.flags_physics    .is_some() as u64) <<  6;
		bitfield |= (creature_update.affiliation      .is_some() as u64) <<  7;
		bitfield |= (creature_update.race             .is_some() as u64) <<  8;
		bitfield |= (creature_update.animation        .is_some() as u64) <<  9;
		bitfield |= (creature_update.animation_time   .is_some() as u64) << 10;
		bitfield |= (creature_update.combo            .is_some() as u64) << 11;
		bitfield |= (creature_update.combo_timeout    .is_some() as u64) << 12;
		bitfield |= (creature_update.appearance       .is_some() as u64) << 13;
		bitfield |= (creature_update.flags            .is_some() as u64) << 14;
		bitfield |= (creature_update.effect_time_dodge.is_some() as u64) << 15;
		bitfield |= (creature_update.effect_time_stun .is_some() as u64) << 16;
		bitfield |= (creature_update.effect_time_fear .is_some() as u64) << 17;
		bitfield |= (creature_update.effect_time_chill.is_some() as u64) << 18;
		bitfield |= (creature_update.effect_time_wind .is_some() as u64) << 19;
		bitfield |= (creature_update.show_patch_time  .is_some() as u64) << 20;
		bitfield |= (creature_update.occupation       .is_some() as u64) << 21;
		bitfield |= (creature_update.specialization   .is_some() as u64) << 22;
		bitfield |= (creature_update.mana_charge      .is_some() as u64) << 23;
		bitfield |= (creature_update.unknown24        .is_some() as u64) << 24;
		bitfield |= (creature_update.unknown25        .is_some() as u64) << 25;
		bitfield |= (creature_update.aim_offset       .is_some() as u64) << 26;
		bitfield |= (creature_update.health           .is_some() as u64) << 27;
		bitfield |= (creature_update.mana             .is_some() as u64) << 28;
		bitfield |= (creature_update.blocking_gauge   .is_some() as u64) << 29;
		bitfield |= (creature_update.multipliers      .is_some() as u64) << 30;
		bitfield |= (creature_update.unknown31        .is_some() as u64) << 31;
		bitfield |= (creature_update.unknown32        .is_some() as u64) << 32;
		bitfield |= (creature_update.level            .is_some() as u64) << 33;
		bitfield |= (creature_update.experience       .is_some() as u64) << 34;
		bitfield |= (creature_update.master           .is_some() as u64) << 35;
		bitfield |= (creature_update.unknown36        .is_some() as u64) << 36;
		bitfield |= (creature_update.rarity           .is_some() as u64) << 37;
		bitfield |= (creature_update.unknown38        .is_some() as u64) << 38;
		bitfield |= (creature_update.home_zone        .is_some() as u64) << 39;
		bitfield |= (creature_update.home             .is_some() as u64) << 40;
		bitfield |= (creature_update.zone_to_reveal   .is_some() as u64) << 41;
		bitfield |= (creature_update.unknown42        .is_some() as u64) << 42;
		bitfield |= (creature_update.consumable       .is_some() as u64) << 43;
		bitfield |= (creature_update.equipment        .is_some() as u64) << 44;
		bitfield |= (creature_update.name             .is_some() as u64) << 45;
		bitfield |= (creature_update.skill_tree       .is_some() as u64) << 46;
		bitfield |= (creature_update.mana_cubes       .is_some() as u64) << 47;

		let mut buffer = vec![];
		{
			let mut encoder = ZlibEncoder::new(&mut buffer);

			encoder.write_arbitrary(&creature_update.id).await?;
			encoder.write_arbitrary(&bitfield).await?;

			//todo: macro
			if let Some(it) = &creature_update.position          { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.rotation          { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.velocity          { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.acceleration      { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.velocity_extra    { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.head_tilt         { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.flags_physics     { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.affiliation       { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.race              { encoder.write_arbitrary(&(*it as i32)).await?; } //see de-serialization
			if let Some(it) = &creature_update.animation         { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.animation_time    { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.combo             { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.combo_timeout     { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.appearance        { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.flags             { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.effect_time_dodge { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.effect_time_stun  { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.effect_time_fear  { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.effect_time_chill { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.effect_time_wind  { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.show_patch_time   { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.occupation        { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.specialization    { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.mana_charge       { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.unknown24         { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.unknown25         { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.aim_offset        { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.health            { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.mana              { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.blocking_gauge    { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.multipliers       { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.unknown31         { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.unknown32         { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.level             { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.experience        { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.master            { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.unknown36         { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.rarity            { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.unknown38         { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.home_zone         { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.home              { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.zone_to_reveal    { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.unknown42         { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.consumable        { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.equipment         { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.name              {
				let bytes = it.as_bytes();
				if bytes.len() > 16 { return Err(InvalidData.into()) }
				encoder.write_all(bytes).await?;
				encoder.write_all(&vec![0u8; 16 - bytes.len()]).await?;
				//todo: check what happens with non-ascii characters
			}
			if let Some(it) = &creature_update.skill_tree        { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.mana_cubes        { encoder.write_arbitrary(it).await?; }

			encoder.shutdown().await?;
		}

		self.write_arbitrary(&(buffer.len() as i32)).await?;
		self.write_all(&buffer).await
	}
}

impl Validate<CreatureUpdate> for Validator {
	fn validate(creature_update: &CreatureUpdate) -> io::Result<()> {
		if let Some(affiliation) = creature_update.affiliation {
			Validator::validate_enum(affiliation)?
		}
		if let Some(race) = creature_update.race {
			Validator::validate_enum(race)?
		}
		if let Some(animation) = creature_update.animation {
			Validator::validate_enum(animation)?
		}
		if let Some(occupation) = creature_update.occupation {
			Validator::validate_enum(occupation)?
		}
		if let Some(specialization) = creature_update.specialization {
			Validator::validate_enum(specialization)?
		}
		if let Some(ref consumable) = creature_update.consumable {
			Validator::validate(consumable)?
		}
		if let Some(ref equipment) = creature_update.equipment {
			for item in equipment.iter() {
				Validator::validate(item)?
			}
		}

		Ok(())
	}
}

#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter)]
pub enum Affiliation {
	Player,
	Enemy,

	NPC = 3,

	Pet = 5,
	Neutral
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter)]
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
	DaggerM2,
	DaggerM1a,
	DaggerM1b,
	FistM2,
	Kick,
	ShootArrow,
	CrossbowM2,
	CrossbowM2Charging,
	BowM2Charging,
	BoomerangThrow,
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
	Unknown050, //BowM2 but slower
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
	UnarmedM2Charging, //also used for DualWieldM2Charging
	Unknown064, //some sort of dualwield charge?
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
	FireExplosionShort,
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
	Sailing,
	Boulder,
	ManaCubePickup,
	Unknown110 //mob attack?
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AppearanceFlag {
	FourLegged,
	CanFly,




	Trainer = 6,

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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CreatureFlag {
	Climbing,

	Aiming = 2,

	Gliding = 4,
	FriendlyFire,
	Sprinting,
	///causes all incoming hits to "miss". only works when this creature is [Affiliation::Enemy]
	Unreachable,

	Lamp = 9,
	Sniping,
}
impl From<CreatureFlag> for u16 {
	fn from(it: CreatureFlag) -> Self {
		it as Self
	}
}

#[repr(i8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter)]
pub enum Occupation {
	None,
	Warrior,
	Ranger,
	Mage,
	Rogue,

	GeneralShopkeep = i8::MIN,
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
#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter)]
pub enum Specialization {
	Default,
	Alternative,
	Witch
}

pub type Multipliers = ArrayWrapper<Multiplier, f32>;

pub type Equipment = ArrayWrapper<Slot, Item>;

pub type SkillTree = ArrayWrapper<Skill, i32>;