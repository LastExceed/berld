use std::io::ErrorKind::InvalidData;

use async_compression::tokio::bufread::ZlibDecoder;
use async_compression::tokio::write::ZlibEncoder;
use nalgebra::Point3;
use rgb::RGB;
use strum::EnumCount;
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
		let size = self.read_u32_le().await? as usize;
		let mut buffer = vec![0_u8; size];
		self.read_exact(&mut buffer).await?;

		let mut decoder = ZlibDecoder::new(buffer.as_slice());

		let id = decoder.read_arbitrary::<CreatureId>().await?;
		let bitfield = decoder.read_u64_le().await?;

		//todo: macro

		#[expect(clippy::if_then_some_else_none, reason = "false positive")]
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
				let padding = decoder.read_arbitrary::<[u8; 3]>().await?;
				if padding != [0_u8; 3] {
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
			consumable        : if bitfield & (1 << 43) > 0 {
				//explicit type annotation as a workaround for https://github.com/rust-lang/rust/issues/108362
				let consumable = <ZlibDecoder<&[u8]> as ReadCwData<Item>>::read_cw_data(&mut decoder).await?;
				Some(consumable)
			} else { None },
			equipment         : if bitfield & (1 << 44) > 0 {
				//custom read/write impl is necessary solely because of formula weirdness :(
				let mut items = Vec::with_capacity(Slot::COUNT);
				for _ in 0..Slot::COUNT {
					//explicit type annotation as a workaround for https://github.com/rust-lang/rust/issues/108362
					items.push(<ZlibDecoder<&[u8]> as ReadCwData<Item>>::read_cw_data(&mut decoder).await?);
				}

				let x: [_; Slot::COUNT] = items.try_into().unwrap();
				Some(x.into())
			} else { None },
			name              : if bitfield & (1 << 45) > 0 {
				let name = decoder
					.read_arbitrary::<[u8; 16]>()
					.await?
					.into_iter()
					.take_while(|byte| *byte != 0)
					.map(char::from)
					.collect();

				Some(name)
			} else { None },
			skill_tree        : if bitfield & (1 << 46) > 0 { Some(decoder.read_arbitrary().await?) } else { None },
			mana_cubes        : if bitfield & (1 << 47) > 0 { Some(decoder.read_arbitrary().await?) } else { None }
		};

		if !matches!(decoder.read_to_end(&mut vec![0_u8; 0]).await, Ok(0)) {
			return Err(InvalidData.into());
		}
		Ok(instance)
	}
}

impl<Writable: AsyncWrite + Unpin> WriteCwData<CreatureUpdate> for Writable {
	#[expect(clippy::identity_op, reason = "<< 0 is an identity_op, but more visually consistent in this case")]
	#[expect(clippy::too_many_lines, reason = "TODO")]
	async fn write_cw_data(&mut self, creature_update: &CreatureUpdate) -> io::Result<()> {
		let mut bitfield = 0_u64;

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
			if let Some(it) = &creature_update.consumable        { encoder.write_cw_data(it).await?; }
			if let Some(it) = &creature_update.equipment         {
				//custom read/write impl is necessary solely because of formula weirdness :(
				for item in it.iter() {
					encoder.write_cw_data(item).await?;
				}
			}
			if let Some(it) = &creature_update.name              {
				let bytes = it.as_bytes();
				if bytes.len() > 16 { return Err(InvalidData.into()) }
				encoder.write_all(bytes).await?;
				encoder.write_all(&vec![0_u8; 16 - bytes.len()]).await?;
			}
			if let Some(it) = &creature_update.skill_tree        { encoder.write_arbitrary(it).await?; }
			if let Some(it) = &creature_update.mana_cubes        { encoder.write_arbitrary(it).await?; }

			encoder.shutdown().await?;
		};

		self.write_i32_le(buffer.len() as _).await?;
		self.write_all(&buffer).await
	}
}

impl Validate<CreatureUpdate> for Validator {
	fn validate(creature_update: &CreatureUpdate) -> io::Result<()> {
		if let Some(affiliation) = creature_update.affiliation {
			Self::validate_enum(&affiliation)?;
		}
		if let Some(race) = creature_update.race {
			Self::validate_enum(&race)?;
		}
		if let Some(animation) = creature_update.animation {
			Self::validate_enum(&animation)?;
		}
		if let Some(occupation) = creature_update.occupation {
			Self::validate_enum(&occupation)?;
		}
		if let Some(specialization) = creature_update.specialization {
			Self::validate_enum(&specialization)?;
		}
		if let Some(ref consumable) = creature_update.consumable {
			Self::validate(consumable)?;
		}
		if let Some(ref equipment) = creature_update.equipment {
			for item in equipment.iter() {
				Self::validate(item)?;
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
	Unknown3,
	CanBreathe,
	PushingWall,
	PushingObject,
	Unknown7//might not exist
}
impl From<PhysicsFlag> for usize {
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
	AltDaggerM1a, //used by humanoids - like daggerM1, but yellow dmg + mana drain
	AltDatterM1b, //used by humanoids - like daggerM1, but yellow dmg + mana drain
	LongswordM2,
	UnarmedM1a, //also used with fists
	UnarmedM1b,
	ShieldM2Charging,
	ShieldM1a,
	ShieldM1b,
	UnarmedM2,
	UnusedDualWieldAttack, //animation is like ripping swords apart
	LongswordM1a,
	LongswordM1b,
	UnusedGreatweapon1, //probably for greatweapon A1
	UnusedDaggerM2, //same as normal DaggerM2, but without poison
	DaggerM2,
	DaggerM1a,
	DaggerM1b,
	FistM2,
	Kick,
	ShootArrow,
	CrossbowM2,
	CrossbowM2Charging,
	BowM2Charging,//also used by snout beetles
	BoomerangThrow,
	BoomerangM2Charging,
	BeamDraining,//used by rune giant
	//29 nothing
	StaffFireM1 = 30,
	StaffFireM2,
	StaffWaterM1,
	StaffWaterM2,
	HealingStream,
	UnusedSummon,
	UnusedCharging1,
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
	UnusedSmash, //same as normal smash, but without jump or damage
	Intercept,
	Teleport,
	UnusedBowM2, //BowM2 but slower
	VolantAttack, //monster default slow attack?
	UnusedIdle, //immediately switches to Idle
	//53 nothing
	Smash = 54,
	BowM2,//also used by snout beetles
	//56 nothing
	GreatweaponM1a = 57,
	GreatweaponM1c,
	GreatweaponM2Charging,
	GreatweaponM2Berserker,
	GreatweaponM2Guardian,
	UnusedStab, //very similar to daggerM1b, but very fast
	UnarmedM2Charging, //also used for DualWieldM2Charging
	UnusedCharging2, //some sort of dualwield charging?
	UnusedGreatweapon2,
	UnusedGreatweapon3,
	GreatweaponM1b,
	Charge1,
	Charge2,
	UnusedSpinkick,
	TurtleBlock,//unused
	TurtleSpin,
	LichScream,
	UnusedStomp,
	QuadrupedAttack,
	ChargeFrontFlip,
	ChargeLeftFlip,
	ChargeRightFlip,
	Stealth,
	Drink,
	Eat,
	PetFoodPresent,
	Sit,
	Sleep,
	//85 nothing
	Cyclone = 86,
	FireExplosionLong,//used by bosses
	FireExplosionShort,
	Lava,//used by bosses
	UnusedSplash,
	EarthQuake,//used by troll
	Clone,
	ChargeM2, //does UnarmedM2, DaggerM2, or GreatweaponM2Guardian during the run, depending on equipped weapons. also applies camouflage-like visual effect. used by werewolves
	FireBeam,//unused, future wand m1
	FireRay,//used by wizards and witches, future wand m2
	Shuriken,
	//97 nothing
	UnusedBlock = 98,//looks different depending on leftweapon slot being empty or not
	//99 nothing
	//100 nothing
	SuperBulwalk = 101, //unused, casts bulwalk
	//102 nothing
	SuperManaShield = 103, //unused - casts manashield
	ShieldM2,
	TeleportToCity,
	Riding,
	Sail,
	Boulder,
	ManaCubePickup,
	UnusedQuadrupedAttack //same as normal QuadrupedAttack except no sound nor damage
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Appearance {
	pub unknown: i16,
	pub hair_color: RGB<u8>,
	//pad1
	pub flags: FlagSet<u16, AppearanceFlag>,
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AppearanceFlag {
	Quadruped,
	Volant,
	Unknown2,//seen on rune giant
	Unknown3,//fast attacks (not humanoid style)? skeleton, all slimes, all runners, peacock, cornling/radishling, imp, mole, biter, coala, squirrel, raccoon, owl, penguin, werewofl, zombie, snout beetle, crab/seacrab, rune giant
	Unknown4,//insect guard, onionling, all tamable
	Unknown5,//town animals, town neutrals, ogre, witch, radishling
	Trainer,
	Unknown7,//might not exist
	Immovable,
	BossGlow,
	EyeGlow,
	Unknown11,//on clones
	DungeonMonster,
	MissionObjective,
	Unknown14,//might not exist
	Unknown15//might not exist
}
impl From<AppearanceFlag> for usize {
	fn from(it: AppearanceFlag) -> Self {
		it as Self
	}
}

#[repr(u16)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CreatureFlag {
	Climbing,
	Unknown1,
	Aiming,
	Unknown3,
	Gliding,
	FriendlyFire,
	Sprinting,
	///causes all incoming hits to "miss". only works when this creature is [Affiliation::Enemy]
	Unreachable,
	Unkown8,
	Lamp,
	Sniping,
	Unknown11,//might not exist
	Unknown12,//might not exist
	Unknown13,//might not exist
	Unknown14,//might not exist
	Unknown15//might not exist
}
impl From<CreatureFlag> for usize {
	fn from(it: CreatureFlag) -> Self {
		it as Self
	}
}

#[repr(i8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter)]
pub enum Specialization {
	Default,
	Alternative,
	Witch
}

pub type Multipliers = ArrayWrapper<Multiplier>;

pub type Equipment = ArrayWrapper<Slot>;

pub type SkillTree = ArrayWrapper<Skill>;