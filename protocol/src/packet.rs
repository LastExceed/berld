use nalgebra::{Point2, Point3, Vector3};

use crate::{bulk_impl, Packet, WriteCwData};
use crate::utils::flagset::{FlagSet16, FlagSet32};
use crate::utils::io_extensions::{ReadArbitrary, WriteArbitrary};

use self::airship_traffic::*;
use self::common::*;
use self::creature_update::*;
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

#[derive(Debug, Clone, Default)]
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
	pub flags_physics: Option<FlagSet32<PhysicsFlag>>,
	pub affiliation: Option<Affiliation>,
	pub race: Option<Race>,
	pub animation: Option<Animation>,
	pub animation_time: Option<i32>,
	pub combo: Option<i32>,
	pub combo_timeout: Option<i32>,
	pub appearance: Option<Appearance>,
	pub flags: Option<FlagSet16<CreatureFlag>>,
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
	pub name: Option<String>,
	pub skill_tree: Option<SkillTree>,
	pub mana_cubes: Option<i32>
}

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct MultiCreatureUpdate; //todo

#[derive(Debug, Clone, Default)]
pub struct AirshipTraffic {
	pub airships: Vec<Airship>
}

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct ServerTick;

#[derive(Debug, Clone, Default)]
pub struct WorldUpdate {
	pub blocks: Vec<Block>,
	pub hits: Vec<Hit>,
	pub particles: Vec<Particle>,
	pub sounds: Vec<Sound>,
	pub projectiles: Vec<Projectile>,
	pub world_objects: Vec<WorldObject>,
	pub drops: Vec<(Point2<i32>, Vec<Drop>)>,//todo: dedicated type
	pub p48s: Vec<P48>,
	pub pickups: Vec<Pickup>,
	pub kills: Vec<Kill>,
	pub attacks: Vec<Attack>,
	pub status_effects: Vec<StatusEffect>,
	pub missions: Vec<Mission>
}

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct IngameDatetime {
	pub day: i32,
	pub time: i32
}

#[repr(C)]
#[derive(Debug, Clone)]//todo: Default
pub struct CreatureAction {
	pub item: Item,
	pub zone: Point2<i32>,
	pub item_index: i32,
	pub unknown_a: i32,
	pub kind: creature_action::Kind
	//pad3
}

#[repr(C)]
#[derive(Debug, Clone, Default)]
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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
	///It is merely used to calculate the heal power of puddles (5% of base_damage)
	pub base_damage: f32,
	pub unknown_b: f32, //2-4 depending on mana for boomerangs, otherwise 0.5
	pub scale: f32,
	pub mana: f32,
	pub particles: f32,
	pub is_yellow: bool,
	//pad3
	pub kind: projectile::Kind,
	pub unknown_c: i32,
	pub unknown_d: f32,
	pub unknown_e: f32
}

#[derive(Debug, Clone, Default)]
pub struct ChatMessageFromClient {
	pub text: String
}
#[derive(Clone)]
pub struct ChatMessageFromServer {
	pub source: CreatureId,
	pub text: String
}

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct ZoneDiscovery(pub Point2<i32>);

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct RegionDiscovery(pub Point2<i32>);

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct MapSeed(pub i32);

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct ConnectionAcceptance;

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct ProtocolVersion(pub i32);

#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct ConnectionRejection;


#[derive(Debug, Eq, PartialEq)]
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
impl Packet for ZoneDiscovery         { const ID: Id = Id(11); }
impl Packet for RegionDiscovery       { const ID: Id = Id(12); }
impl Packet for MapSeed               { const ID: Id = Id(15); } //to this day 13 and 14 have never been discovered
impl Packet for ConnectionAcceptance  { const ID: Id = Id(16); }
impl Packet for ProtocolVersion       { const ID: Id = Id(17); }
impl Packet for ConnectionRejection   { const ID: Id = Id(18); }

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
	ZoneDiscovery,
	RegionDiscovery,
	ProtocolVersion
);