use std::collections::HashMap;
use nalgebra::{Point2, Point3, Vector3};

use crate::{bulk_impl, Packet, WriteCwData};
use crate::packet::area_request::{Area, Region, Zone};
use crate::packet::world_update::p48::P48sub;
use crate::utils::flagset::FlagSet;
use crate::utils::io_extensions::{ReadArbitrary as _, WriteArbitrary as _};

use self::airship_traffic::*;
use self::common::*;
use self::creature_update::*;
use self::world_update::*;
use self::world_update::loot::GroundItem;

pub mod creature_update;
pub mod airship_traffic;
pub mod world_update;
pub mod creature_action;
pub mod hit;
pub mod status_effect;
pub mod projectile;
pub mod chat_message;
pub mod common;
pub mod area_request;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct CreatureUpdate {
	pub id: CreatureId,
	pub position: Option<Point3<i64>>,
	pub rotation: Option<EulerAngles>,
	pub velocity: Option<Vector3<f32>>,
	pub acceleration: Option<Vector3<f32>>,
	///used by the 'retreat' ability
	pub velocity_extra: Option<Vector3<f32>>,
	///used for climbing, vertical attacking, stuns, respawns, and maybe more
	pub head_tilt: Option<f32>,
	pub flags_physics: Option<FlagSet<u32, PhysicsFlag>>,
	pub affiliation: Option<Affiliation>,
	pub race: Option<Race>,
	pub animation: Option<Animation>,
	pub animation_time: Option<i32>,
	pub combo: Option<i32>,
	pub combo_timeout: Option<i32>,
	pub appearance: Option<Appearance>,
	pub flags: Option<FlagSet<u16, CreatureFlag>>,
	pub effect_time_dodge: Option<i32>,
	pub effect_time_stun: Option<i32>,
	pub effect_time_fear: Option<i32>,
	pub effect_time_chill: Option<i32>,
	pub effect_time_wind: Option<i32>,
	///unknown purpose, name adopted from cuwo
	pub show_patch_time: Option<i32>,
	pub occupation: Option<Occupation>,
	pub specialization: Option<Specialization>,
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
	pub rarity: Option<u8>,
	pub unknown38: Option<i32>,
	pub home_zone: Option<Point3<i32>>,
	pub home: Option<Point3<i64>>,
	///players within ±2 [level] of the dungeon in this zone see a green speech bubble above this creature, and can get this zone revealed on the map by talking to this creature
	pub zone_to_reveal: Option<Point3<i32>>,
	pub unknown42: Option<i8>, //todo: 0 3 4 for villages - 3 = dialog about pet food
	pub consumable: Option<Item>,
	pub equipment: Option<Equipment>,
	pub name: Option<String>, //todo: AsciiString
	pub skill_tree: Option<SkillTree>,
	pub mana_cubes: Option<i32>
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct MultiCreatureUpdate; //todo

#[derive(Debug, PartialEq, Clone, Default)]
pub struct AirshipTraffic {
	pub airships: Vec<Airship>
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct ServerTick;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct WorldUpdate {
	pub blocks: Vec<Block>,
	pub hits: Vec<Hit>,
	pub particles: Vec<Particle>,
	pub sounds: Vec<Sound>,
	pub projectiles: Vec<Projectile>,
	pub world_objects: Vec<WorldObject>,
	pub loot: HashMap<Point2<i32>, Vec<GroundItem>>,
	pub p48: HashMap<Point2<i32>, Vec<P48sub>>,
	pub pickups: Vec<Pickup>,
	pub kills: Vec<Kill>,
	pub attacks: Vec<Attack>,
	pub status_effects: Vec<StatusEffect>,
	pub missions: Vec<Mission>
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct IngameDatetime {
	pub day: i32,
	pub time: i32
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]//todo: Default
pub struct CreatureAction {
	pub item: Item,
	pub zone: Point2<i32>,
	pub item_index: i32,
	pub unknown_a: i32,
	pub kind: creature_action::Kind //definitely u8
	//pad3
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Hit {
	pub attacker: CreatureId,
	pub target: CreatureId,
	pub damage: f32,
	pub critical: bool,
	//pad3
	pub stuntime: i32,
	//pad4
	pub position: Point3<i64>,
	pub direction: Vector3<f32>,
	pub is_yellow: bool, //u8 used skill according to cuwo
	pub kind: hit::Kind,
	pub flash: bool,
	//pad1
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct StatusEffect {
	pub source: CreatureId,
	pub target: CreatureId,
	pub kind: status_effect::Kind,
	//pad3
	pub modifier: f32,
	pub duration: i32,
	//pad4
	pub creature_id3: CreatureId //=source for poison, else 0
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Projectile {
	pub attacker: u64,
	pub zone: Point2<i32>,
	pub unknown_a: i32,
	//pad4
	pub position: Point3<i64>,
	pub unknown_v: [i32; 3],
	pub velocity: Vector3<f32>,
	///This is NOT the damage that the target will receive (refer to [Hit] for that).
	///
	///It is merely used to calculate the heal power of puddles (5% of [`Projectile::base_damage`])
	pub base_damage: f32,
	pub unknown_b: f32, //2-4 depending on mana for boomerangs, otherwise 0.5
	pub scale: f32,
	pub mana: f32,
	pub particles: f32,
	pub is_yellow: bool,
	//pad3
	pub kind: projectile::Kind,
	//pad4 - contains uninit memory
	pub unknown_c: i64 //always 0 ?
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct ChatMessageFromClient {
	pub text: String
}
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct ChatMessageFromServer {
	pub source: CreatureId,
	pub text: String
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct AreaRequest<A: Area>(pub Point2<A::Coordinate>);

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct MapSeed(pub i32);

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct ConnectionAcceptance;

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct ProtocolVersion(pub i32);

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct ConnectionRejection;


#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Id(i32);
//the anonymous field is intentionally kept private to prevent manual construction
//serialization isnt affected as it constructs this via transmutation

//todo: macro
impl Packet for CreatureUpdate        { const ID: Id = Id( 0); }
impl Packet for MultiCreatureUpdate   { const ID: Id = Id( 1); }
impl Packet for ServerTick            { const ID: Id = Id( 2); }
impl Packet for AirshipTraffic        { const ID: Id = Id( 3); }
impl Packet for WorldUpdate           { const ID: Id = Id( 4); }
impl Packet for IngameDatetime        { const ID: Id = Id( 5); }
impl Packet for CreatureAction        { const ID: Id = Id( 6); }
impl Packet for Hit                   { const ID: Id = Id( 7); }
impl Packet for StatusEffect          { const ID: Id = Id( 8); }
impl Packet for Projectile            { const ID: Id = Id( 9); }
impl Packet for ChatMessageFromClient { const ID: Id = Id(10); }
impl Packet for ChatMessageFromServer { const ID: Id = Id(10); }
impl Packet for AreaRequest<Zone>     { const ID: Id = Id(11); }
impl Packet for AreaRequest<Region>   { const ID: Id = Id(12); }
impl Packet for MapSeed               { const ID: Id = Id(15); }
impl Packet for ConnectionAcceptance  { const ID: Id = Id(16); }
impl Packet for ProtocolVersion       { const ID: Id = Id(17); }
impl Packet for ConnectionRejection   { const ID: Id = Id(18); }
//to this day [Id] 13 and 14 have never been discovered
//if they do exist, then they must be either 0 sized or C->S only (or both)

//these are just for type safety to prevent sending packets in the wrong direction
pub trait FromServer: Packet {}
pub trait FromClient: Packet {}

bulk_impl!(FromServer for
	CreatureUpdate,
	MultiCreatureUpdate,
	ServerTick,
	AirshipTraffic,
	WorldUpdate,
	IngameDatetime,
	ChatMessageFromServer,
	MapSeed,
	ConnectionAcceptance,
	ProtocolVersion,
	ConnectionRejection
);

bulk_impl!(FromClient for
	CreatureUpdate,
	CreatureAction,
	Hit,
	StatusEffect,
	Projectile,
	ChatMessageFromClient,
	AreaRequest<Zone>,
	AreaRequest<Region>,
	ProtocolVersion
);