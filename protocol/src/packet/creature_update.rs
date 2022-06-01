use std::ffi::CStr;
use std::io::{Error, ErrorKind};
use bitvec::order::Lsb0;
use bitvec::view::BitView;
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use crate::packet::*;
use crate::utils::ReadExtension;

pub type PhysicsFlags = u32;
pub type CreatureFlags = u16;
pub type AppearanceFlags = u16;

#[derive(Default)]
pub struct CreatureUpdate {
	pub id: CreatureId,
	pub position: Option<Point<i64, 3>>,
	pub rotation: Option<[f32; 3]>,//todo: type
	pub velocity: Option<Vector3<f32>>,
	pub acceleration: Option<Vector3<f32>>,
	/**used by the 'retreat' ability*/
	pub velocity_extra: Option<Vector3<f32>>,
	pub climb_animation_state: Option<f32>,
	pub flags_physics: Option<PhysicsFlags>,
	pub affiliation: Option<Affiliation>,
	pub race: Option<Race>,
	pub animation: Option<Animation>,
	pub animation_time: Option<i32>,
	pub combo: Option<i32>,
	pub hit_time_out: Option<i32>,
	pub appearance: Option<Appearance>,
	pub flags: Option<CreatureFlags>,
	pub effect_time_dodge: Option<i32>,
	pub effect_time_stun: Option<i32>,
	pub effect_time_fear: Option<i32>,
	pub effect_time_ice: Option<i32>,
	pub effect_time_wind: Option<i32>,
	/**unknown purpose>, name adopted from cuwo*/
	pub show_patch_time: Option<i32>,
	pub combat_class_major: Option<CombatClassMajor>,
	pub combat_class_minor: Option<CombatClassMinor>,
	pub mana_charge: Option<f32>,
	pub unknown24: Option<[f32; 3]>,
	pub unknown25: Option<[f32; 3]>,
	/**coordinates of the location this creature is aiming at>, relative to its own position*/
	pub aim_displacement: Option<Point<f32, 3>>,
	pub health: Option<f32>,
	pub mana: Option<f32>,
	pub blocking_gauge: Option<f32>,
	pub multipliers: Option<Multipliers>,
	pub unknown31: Option<i8>,
	pub unknown32: Option<i8>,
	pub level: Option<i32>,
	pub experience: Option<i32>,
	/**for pets this is the [CreatureId] of their owner*/
	pub master: Option<CreatureId>,
	pub unknown36: Option<i64>,
	/**this is the '+#' that monsters in some dungeons have next to their [race]*/
	pub power_base: Option<i8>,
	pub unknown38: Option<i32>,
	pub home_chunk: Option<Point<i32, 3>>,
	pub home: Option<Point<i64, 3>>,
	/**players within ±2 [level] of the dungeon at these coordinates see a green speech bubble above this creature's head and can get that chunk revealed on the map by talking to this creature*/
	pub chunk_to_reveal: Option<Point<i32, 3>>,
	pub unknown42: Option<i8>,//0 3 4 for villages - 3 = dialog about pet food
	pub consumable: Option<Item>,
	pub equipment: Option<Equipment>,
	pub name: Option<String>,
	pub skill_tree: Option<SkillTree>,
	pub mana_cubes: Option<i32>
}

impl CwSerializable for CreatureUpdate {
	fn read_from<T: Read>(reader: &mut T) -> Result<Self, Error> where T: Read {
		//todo: can't decode from network stream directly because ???
		let size = reader.read_struct::<i32>()?;
		let mut buffer = vec![0u8; size as usize];
		reader.read_exact(&mut buffer)?;

		let mut decoder = Box::new(ZlibDecoder::new(buffer.as_slice())) as Box<dyn Read>; //todo: this cant be right

		let id = decoder.read_struct::<CreatureId>()?;

		let bitfield_buffer = decoder.read_struct::<u64>()?;
		let mut bitfield = bitfield_buffer.view_bits::<Lsb0>().iter();

		//todo: macro
		let instance = Self {
			id,
			position             : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			rotation             : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			velocity             : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			acceleration         : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			velocity_extra       : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			climb_animation_state: if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			flags_physics        : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			affiliation          : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			race                 : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			animation            : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			animation_time       : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			combo                : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			hit_time_out         : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			appearance           : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			flags                : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			effect_time_dodge    : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			effect_time_stun     : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			effect_time_fear     : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			effect_time_ice      : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			effect_time_wind     : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			show_patch_time      : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			combat_class_major   : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			combat_class_minor   : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			mana_charge          : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			unknown24            : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			unknown25            : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			aim_displacement     : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			health               : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			mana                 : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			blocking_gauge       : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			multipliers          : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			unknown31            : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			unknown32            : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			level                : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			experience           : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			master               : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			unknown36            : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			power_base           : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			unknown38            : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			home_chunk           : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			home                 : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			chunk_to_reveal      : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			unknown42            : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			consumable           : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			equipment            : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			name                 : if *bitfield.next().unwrap() {
				if let Ok(cstr) = CStr::from_bytes_until_nul(decoder.read_struct::<[u8; 16]>()?.as_slice()) {
					Some(cstr.to_str().unwrap().to_string())
				} else {
					return Err(Error::from(ErrorKind::InvalidData));
				}
			} else { None },
			skill_tree           : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None },
			mana_cubes           : if *bitfield.next().unwrap() { Some(decoder.read_struct()?) } else { None }
		};
		assert!(matches!(decoder.read_to_end(&mut vec![0u8; 0]), Ok(0))); //todo: replace panic with error
		Ok(instance)
	}

	fn write_to<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
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
		bitfield |= (self.effect_time_ice      .is_some() as u64) << 18;
		bitfield |= (self.effect_time_wind     .is_some() as u64) << 19;
		bitfield |= (self.show_patch_time      .is_some() as u64) << 20;
		bitfield |= (self.combat_class_major   .is_some() as u64) << 21;
		bitfield |= (self.combat_class_minor   .is_some() as u64) << 22;
		bitfield |= (self.mana_charge          .is_some() as u64) << 23;
		bitfield |= (self.unknown24            .is_some() as u64) << 24;
		bitfield |= (self.unknown25            .is_some() as u64) << 25;
		bitfield |= (self.aim_displacement     .is_some() as u64) << 26;
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
		bitfield |= (self.home_chunk           .is_some() as u64) << 39;
		bitfield |= (self.home                 .is_some() as u64) << 40;
		bitfield |= (self.chunk_to_reveal      .is_some() as u64) << 41;
		bitfield |= (self.unknown42            .is_some() as u64) << 42;
		bitfield |= (self.consumable           .is_some() as u64) << 43;
		bitfield |= (self.equipment            .is_some() as u64) << 44;
		bitfield |= (self.name                 .is_some() as u64) << 45;
		bitfield |= (self.skill_tree           .is_some() as u64) << 46;
		bitfield |= (self.mana_cubes           .is_some() as u64) << 47;

		let mut buffer = Vec::new();
		{
			let mut encoder = Box::new(ZlibEncoder::new(&mut buffer, Compression::default())) as Box<dyn Write>;

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
			if let Some(it) = &self.effect_time_ice       { encoder.write_struct(it)?; }
			if let Some(it) = &self.effect_time_wind      { encoder.write_struct(it)?; }
			if let Some(it) = &self.show_patch_time       { encoder.write_struct(it)?; }
			if let Some(it) = &self.combat_class_major    { encoder.write_struct(it)?; }
			if let Some(it) = &self.combat_class_minor    { encoder.write_struct(it)?; }
			if let Some(it) = &self.mana_charge           { encoder.write_struct(it)?; }
			if let Some(it) = &self.unknown24             { encoder.write_struct(it)?; }
			if let Some(it) = &self.unknown25             { encoder.write_struct(it)?; }
			if let Some(it) = &self.aim_displacement      { encoder.write_struct(it)?; }
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
			if let Some(it) = &self.home_chunk            { encoder.write_struct(it)?; }
			if let Some(it) = &self.home                  { encoder.write_struct(it)?; }
			if let Some(it) = &self.chunk_to_reveal       { encoder.write_struct(it)?; }
			if let Some(it) = &self.unknown42             { encoder.write_struct(it)?; }
			if let Some(it) = &self.consumable            { encoder.write_struct(it)?; }
			if let Some(it) = &self.equipment             { encoder.write_struct(it)?; }
			if let Some(it) = &self.name                  {
				//todo: there gotta be a better way to do this
				let mut buf = it.chars().take(16).map(|c|{c as u8}).collect::<Vec<u8>>();
				for _ in 0..(16 - buf.len()) {
					buf.push(0);
				}
				encoder.write_all(buf.as_slice())?;
			}
			if let Some(it) = &self.skill_tree            { encoder.write_struct(it)?; }
			if let Some(it) = &self.mana_cubes            { encoder.write_struct(it)?; }

			encoder.flush()?;
		}

		writer.write_struct(&(buffer.len() as i32))?;
		writer.write_all(&buffer)
	}
}
impl Packet for CreatureUpdate {
	fn id() -> PacketId {
		PacketId::CreatureUpdate
	}
}

#[derive(Default, Clone, Copy)]
pub struct CreatureId(pub i64);

#[repr(u8)]
pub enum Affiliation {
	Player,
	Enemy,

	NPC = 3,

	Pet = 5,
	Neutral
}

#[repr(i32)]
pub enum Race {
	ElfMale,
	ElfFemale,
	HumanMale,
	HumanFemale,
	GoblinMale,
	GoblinFemale,
	TerrierBull,
	LizardmanMale,
	LizardmanFemale,
	DwarfMale,
	DwarfFemale,
	OrcMale,
	OrcFemale,
	FrogmanMale,
	FrogmanFemale,
	UndeadMale,
	UndeadFemale,
	Skeleton,
	OldMan,
	Collie,
	ShepherdDog,
	SkullBull,
	Alpaca,
	AlpacaBrown,
	Egg,
	Turtle,
	Terrier,
	TerrierScottish,
	Wolf,
	Panther,
	Cat,
	CatBrown,
	CatWhite,
	Pig,
	Sheep,
	Bunny,
	Porcupine,
	SlimeGreen,
	SlimePink,
	SlimeYellow,
	SlimeBlue,
	Frightener,
	Sandhorror,
	Wizard,
	Bandit,
	Witch,
	Ogre,
	Rockling,
	Gnoll,
	GnollPolar,
	Monkey,
	Gnobold,
	Insectoid,
	Hornet,
	InsectGuard,
	Crow,
	Chicken,
	Seagull,
	Parrot,
	Bat,
	Fly,
	Midge,
	Mosquito,
	RunnerPlain,
	RunnerLeaf,
	RunnerSnow,
	RunnerDesert,
	Peacock,
	Frog,
	CreaturePlant,
	CreatureRadish,
	Onionling,
	OnionlingDesert,
	Devourer,
	Duckbill,
	Crocodile,
	CreatureSpike,
	Anubis,
	Horus,
	Jester,
	Spectrino,
	Djinn,
	Minotaur,
	NomadMale,
	NomadFemale,
	Imp,
	Spitter,
	Mole,
	Biter,
	Koala,
	Squirrel,
	Raccoon,
	Owl,
	Penguin,
	Werewolf,
	Santa,
	Zombie,
	Vampire,
	Horse,
	Camel,
	Cow,
	Dragon,
	BeetleDark,
	BeetleFire,
	BeetleSnout,
	BeetleLemon,
	Crab,
	CrabSea,
	Troll,
	TrollDark,
	Helldemon,
	Golem,
	GolemEmber,
	GolemSnow,
	Yeti,
	Cyclops,
	Mammoth,
	Lich,
	Runegiant,
	Saurian,
	Bush,
	BushSnow,
	BushSnowberry,
	PlantCotton,
	Scrub,
	ScrubCobweg,
	ScrubFire,
	Ginseng,
	Cactus,
	ChristmasTree,
	Thorntree,
	DepositGold,
	DepositIron,
	DepositSilver,
	DepositSandstone,
	DepositEmerald,
	DepositSapphire,
	DepositRuby,
	DepositDiamond,
	DepositIcecrystal,
	Scarecrow,
	Aim,
	Dummy,
	Vase,
	Bomb,
	FishSapphire,
	FishLemon,
	Seahorse,
	Mermaid,
	Merman,
	Shark,
	Bumblebee,
	Lanternfish,
	Mawfish,
	Piranha,
	Blowfish
}

#[repr(u8)]
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
pub struct Appearance {
	pub unknown: i16,
	pub hair_color: [u8; 3],//todo: type
	//pad1
	pub flags: AppearanceFlags,
	pub creature_size: [f32; 3],//todo: type
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
	pub hand_rotation: [f32; 3],//todo: type
	pub feet_rotation: f32,
	pub wing_rotation: f32,
	pub tail_rotation: f32,
	pub body_offset: Point<f32, 3>,
	pub head_offset: Point<f32, 3>,
	pub hand_offset: Point<f32, 3>,
	pub foot_offset: Point<f32, 3>,
	pub tail_offset: Point<f32, 3>,
	pub wing_offset: Point<f32, 3>
}

pub struct AppearanceFlag {

}

#[repr(i8)]
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
pub enum CombatClassMinor {
	Default,
	Alternative,
	Witch
}

#[repr(C)]
pub struct Multipliers {
	pub health: f32,
	pub attack_speed: f32,
	pub damage: f32,
	pub armor: f32,
	pub resi: f32
}

#[repr(C)]
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