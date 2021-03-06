use std::io::{Error, Read, Write};
use std::mem::size_of;

use nalgebra::{Point2, Point3, Vector3};

use crate::utils::flagset::{FlagSet16, FlagSet32};
use crate::utils::io_extensions::{ReadExtension, WriteExtension};

use self::airship_traffic::*;
use self::common::*;
use self::creature_action::*;
use self::creature_update::*;
use self::hit::*;
use self::projectile::*;
use self::status_effect::*;
use self::world_update::*;
use self::world_update::drops::Drop;

pub mod creature_update;
pub mod airship_traffic;
pub mod world_update;
pub mod creature_action;
pub mod hit;
pub mod status_effect;
pub mod projectile;
pub mod chat_message;
pub mod common;

#[derive(Default)]
pub struct CreatureUpdate {
	pub id: CreatureId,
	pub position: Option<Point3<i64>>,
	pub rotation: Option<EulerAngles>,
	pub velocity: Option<Vector3<f32>>,
	pub acceleration: Option<Vector3<f32>>,
	///used by the 'retreat' ability
	pub velocity_extra: Option<Vector3<f32>>,
	pub climb_animation_state: Option<f32>,
	pub flags_physics: Option<FlagSet32<PhysicsFlag>>,
	pub affiliation: Option<Affiliation>,
	pub race: Option<Race>,
	pub animation: Option<Animation>,
	pub animation_time: Option<i32>,
	pub combo: Option<i32>,
	pub hit_time_out: Option<i32>,
	pub appearance: Option<Appearance>,
	pub flags: Option<FlagSet16<CreatureFlag>>,
	pub effect_time_dodge: Option<i32>,
	pub effect_time_stun: Option<i32>,
	pub effect_time_fear: Option<i32>,
	pub effect_time_chill: Option<i32>,
	pub effect_time_wind: Option<i32>,
	///unknown purpose, name adopted from cuwo
	pub show_patch_time: Option<i32>,
	pub combat_class_major: Option<CombatClassMajor>,
	pub combat_class_minor: Option<CombatClassMinor>,
	pub mana_charge: Option<f32>,
	pub unknown24: Option<[f32; 3]>,
	pub unknown25: Option<[f32; 3]>,
	///coordinates of the location this creature is aiming at>, relative to its own position
	pub aim_offset: Option<Point3<f32>>,
	pub health: Option<f32>,
	pub mana: Option<f32>,
	pub blocking_gauge: Option<f32>,
	pub multipliers: Option<Multipliers>,
	pub unknown31: Option<i8>,
	pub unknown32: Option<i8>,
	pub level: Option<i32>,
	pub experience: Option<i32>,
	///for pets this is the [id] of their owner
	pub master: Option<CreatureId>,
	pub unknown36: Option<i64>,
	///this is the '+#' that monsters in some dungeons have next to their [race]
	pub power_base: Option<i8>,
	pub unknown38: Option<i32>,
	pub home_zone: Option<Point3<i32>>,
	pub home: Option<Point3<i64>>,
	///players within ??2 [level] of the dungeon in this zone see a green speech bubble above this creature, and can get this zone revealed on the map by talking to this creature
	pub zone_to_reveal: Option<Point3<i32>>,
	pub unknown42: Option<i8>, //todo: 0 3 4 for villages - 3 = dialog about pet food
	pub consumable: Option<Item>,
	pub equipment: Option<Equipment>,
	pub name: Option<String>,
	pub skill_tree: Option<SkillTree>,
	pub mana_cubes: Option<i32>
}

#[repr(C)]
pub struct MultiCreatureUpdate; //todo

pub struct AirshipTraffic {
	pub airships: Vec<Airship>
}

#[repr(C)]
pub struct ServerTick;

#[derive(Default)]
pub struct WorldUpdate {
	pub world_edits: Vec<WorldEdit>,
	pub hits: Vec<Hit>,
	pub particles: Vec<Particle>,
	pub sound_effects: Vec<SoundEffect>,
	pub projectiles: Vec<Projectile>,
	pub world_objects: Vec<WorldObject>,
	pub drops: Vec<(Point2<i32>, Vec<Drop>)>,
	pub p48s: Vec<P48>,
	pub pickups: Vec<Pickup>,
	pub kills: Vec<Kill>,
	pub attacks: Vec<Attack>,
	pub status_effects: Vec<StatusEffect>,
	pub missions: Vec<Mission>
}

#[repr(C)]
pub struct IngameDatetime {
	pub day: i32,
	pub time: i32
}

#[repr(C)]
pub struct CreatureAction {
	pub item: Item,
	pub zone: Point2<i32>,
	pub item_index: i32,
	pub unknown_a: i32,
	pub type_: CreatureActionType
	//pad3
}

#[repr(C)]
pub struct Hit {
	pub attacker: CreatureId,
	pub target: CreatureId,
	pub damage: f32,
	pub critical: bool,
	//pad3
	pub stuntime: i32,
	//pad3
	pub position: Point3<i64>,
	pub direction: Vector3<f32>,
	pub is_yellow: bool,
	pub type_: HitType,
	pub flash: bool,
	//pad1
}

#[repr(C)]
pub struct StatusEffect {
	pub source: CreatureId,
	pub target: CreatureId,
	pub type_: StatusEffectType,
	//pad3
	pub modifier: f32,
	pub duration: i32,
	//pad4
	pub creature_id3: CreatureId
}

#[repr(C)]
pub struct Projectile {
	pub attacker: u64,
	pub zone: Point2<i32>,
	pub unknown_a: i32,
	//pad4
	pub position: Point3<i64>,
	pub unknown_v: [i32; 3],
	pub velocity: Vector3<f32>,
	pub legacy_damage: f32,
	pub unknown_b: f32, //2-4 depending on mana for boomerangs, otherwise 0.5
	pub scale: f32,
	pub mana: f32,
	pub particles: f32,
	pub skill: u8,
	//pad3
	pub type_: ProjectileType,
	pub unknown_c: i32,
	pub unknown_d: f32,
	pub unknown_e: f32
}

pub struct ChatMessageFromClient {
	pub text: String
}
pub struct ChatMessageFromServer {
	pub source: CreatureId,
	pub text: String
}

#[repr(C)]
pub struct ZoneDiscovery(pub Point2<i32>);

#[repr(C)]
pub struct RegionDiscovery(pub Point2<i32>);

#[repr(C)]
pub struct MapSeed(pub i32);

#[repr(C)]
pub struct ConnectionAcceptance;

#[repr(C)]
pub struct ProtocolVersion(pub i32);

#[repr(C)]
pub struct ConnectionRejection;



pub trait CwSerializable: Sized {
	fn read_from(reader: &mut impl Read) -> Result<Self, Error>
		where [(); size_of::<Self>()]:
	{
		reader.read_struct::<Self>()
	}

	fn write_to(&self, writer: &mut impl Write) -> Result<(), Error>
		where [(); size_of::<Self>()]:
	{
		writer.write_struct(self)
	}
}

impl<Element: CwSerializable> CwSerializable for Vec<Element>
	where [(); size_of::<Element>()]:
{
	fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
		(0..reader.read_struct::<i32>()?)
			.map(|_| Element::read_from(reader))
			.collect::<Result<Self, Error>>()
	}

	fn write_to(&self, writer: &mut impl Write) -> Result<(), Error> {
		writer.write_struct(&(self.len() as i32))?;
		for element in self {
			element.write_to(writer)?;
		}
		Ok(())
	}
}

pub trait Packet: CwSerializable {
	const ID: PacketId;

	fn write_to_with_id(&self, writer: &mut impl Write) -> Result<(), Error>
		where [(); size_of::<Self>()]:
	{
		writer.write_struct(&Self::ID)?;
		self.write_to(writer)
	}
}

pub trait PacketFromServer: Packet {}
pub trait PacketFromClient: Packet {}

impl CwSerializable for MultiCreatureUpdate {}
impl CwSerializable for ServerTick {}
impl CwSerializable for IngameDatetime {}
impl CwSerializable for CreatureAction {}
impl CwSerializable for Hit {}
impl CwSerializable for StatusEffect {}
impl CwSerializable for Projectile {}
impl CwSerializable for ZoneDiscovery {}
impl CwSerializable for RegionDiscovery {}
impl CwSerializable for MapSeed {}
impl CwSerializable for ConnectionAcceptance {}
impl CwSerializable for ProtocolVersion {}
impl CwSerializable for ConnectionRejection {}
// packets with non-default trait implementations:
// - CreatureUpdate
// - AirshipTraffic
// - WorldUpdate
// - ChatMessage (both variants)
// they can be found in their respective modules

impl Packet for CreatureUpdate        { const ID: PacketId = PacketId::CreatureUpdate; }
impl Packet for MultiCreatureUpdate   { const ID: PacketId = PacketId::MultiCreatureUpdate; }
impl Packet for ServerTick            { const ID: PacketId = PacketId::ServerTick; }
impl Packet for AirshipTraffic        { const ID: PacketId = PacketId::AirshipTraffic; }
impl Packet for WorldUpdate           { const ID: PacketId = PacketId::WorldUpdate; }
impl Packet for IngameDatetime        { const ID: PacketId = PacketId::IngameDatetime; }
impl Packet for CreatureAction        { const ID: PacketId = PacketId::CreatureAction; }
impl Packet for Hit                   { const ID: PacketId = PacketId::Hit; }
impl Packet for StatusEffect          { const ID: PacketId = PacketId::StatusEffect; }
impl Packet for Projectile            { const ID: PacketId = PacketId::Projectile; }
impl Packet for ChatMessageFromClient { const ID: PacketId = PacketId::ChatMessage; }
impl Packet for ChatMessageFromServer { const ID: PacketId = PacketId::ChatMessage; }
impl Packet for ZoneDiscovery         { const ID: PacketId = PacketId::ZoneDiscovery; }
impl Packet for RegionDiscovery       { const ID: PacketId = PacketId::RegionDiscovery; }
impl Packet for MapSeed               { const ID: PacketId = PacketId::MapSeed; }
impl Packet for ConnectionAcceptance  { const ID: PacketId = PacketId::ConnectionAcceptance; }
impl Packet for ProtocolVersion       { const ID: PacketId = PacketId::ProtocolVersion; }
impl Packet for ConnectionRejection   { const ID: PacketId = PacketId::ConnectionRejection; }

impl PacketFromServer for CreatureUpdate {}
impl PacketFromServer for MultiCreatureUpdate {}
impl PacketFromServer for ServerTick {}
impl PacketFromServer for AirshipTraffic {}
impl PacketFromServer for WorldUpdate {}
impl PacketFromServer for IngameDatetime {}
impl PacketFromServer for ChatMessageFromServer {}
impl PacketFromServer for MapSeed {}
impl PacketFromServer for ConnectionAcceptance {}
impl PacketFromServer for ProtocolVersion {}
impl PacketFromServer for ConnectionRejection {}

impl PacketFromClient for CreatureUpdate {}
impl PacketFromClient for CreatureAction {}
impl PacketFromClient for Hit {}
impl PacketFromClient for StatusEffect {}
impl PacketFromClient for Projectile {}
impl PacketFromClient for ChatMessageFromClient {}
impl PacketFromClient for ZoneDiscovery {}
impl PacketFromClient for RegionDiscovery {}
impl PacketFromClient for ProtocolVersion {}

// this could easily be done with macros
// but that would make things harder to comprehend
// and doesn't provide any meaningful benefit