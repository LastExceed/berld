use std::io::{Error, Read, Write};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use nalgebra::{Point2, Point3, Vector3};
use rgb::{RGB, RGBA};

use crate::bulk_impl;
use crate::packet::{CwSerializable, WorldUpdate};
use crate::packet::common::{CreatureId, Hitbox, Item, Race};
use crate::utils::io_extensions::{ReadExtension, WriteExtension};

use self::mission::*;
use self::p48::*;
use self::particle::*;
use self::sound_effect::*;
use self::world_edit::*;
use self::world_object::*;

pub mod world_edit;
pub mod particle;
pub mod sound_effect;
pub mod world_object;
pub mod drops;
pub mod p48;
pub mod mission;

impl CwSerializable for WorldUpdate {
	fn read_from(readable: &mut impl Read) -> Result<Self, Error> {
		//todo: deduplicate (creature_update)
		let mut buffer = vec![0u8; readable.read_struct::<i32>()? as usize];
		readable.read_exact(&mut buffer)?;

		let mut decoder = ZlibDecoder::new(buffer.as_slice());

		//todo: copypasta
		Ok(Self {
			world_edits   : Vec::read_from(&mut decoder)?,
			hits          : Vec::read_from(&mut decoder)?,
			particles     : Vec::read_from(&mut decoder)?,
			sound_effects : Vec::read_from(&mut decoder)?,
			projectiles   : Vec::read_from(&mut decoder)?,
			world_objects : Vec::read_from(&mut decoder)?,
			drops         : Vec::read_from(&mut decoder)?,
			p48s          : Vec::read_from(&mut decoder)?,
			pickups       : Vec::read_from(&mut decoder)?,
			kills         : Vec::read_from(&mut decoder)?,
			attacks       : Vec::read_from(&mut decoder)?,
			status_effects: Vec::read_from(&mut decoder)?,
			missions      : Vec::read_from(&mut decoder)?
		})
	}

	fn write_to(&self, writable: &mut impl Write) -> Result<(), Error> {
		let mut buffer = vec![];
		{
			let mut encoder = ZlibEncoder::new(&mut buffer, Compression::default());

			//todo: copypasta
			self.world_edits   .write_to(&mut encoder)?;
			self.hits          .write_to(&mut encoder)?;
			self.particles     .write_to(&mut encoder)?;
			self.sound_effects .write_to(&mut encoder)?;
			self.projectiles   .write_to(&mut encoder)?;
			self.world_objects .write_to(&mut encoder)?;
			self.drops         .write_to(&mut encoder)?;
			self.p48s          .write_to(&mut encoder)?;
			self.pickups       .write_to(&mut encoder)?;
			self.kills         .write_to(&mut encoder)?;
			self.attacks       .write_to(&mut encoder)?;
			self.status_effects.write_to(&mut encoder)?;
			self.missions      .write_to(&mut encoder)?;

			encoder.flush()?;
		}
		writable.write_struct(&(buffer.len() as i32))?;
		writable.write_all(&buffer)
	}
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct WorldEdit {
	pub position: Point3<i32>,
	pub color: RGB<u8>,
	pub block_type: BlockType,
	pub padding: i32
}

#[repr(C)]
#[derive(Clone, PartialEq)]
pub struct Particle {
	pub position: Point3<i64>,
	pub velocity: Vector3<f32>,
	pub color: RGBA<f32>,
	pub size: f32,
	pub count: i32,
	pub type_: ParticleType,
	pub spread: f32,
	//pad4
}

#[repr(C)]
#[derive(Clone, PartialEq)]
pub struct SoundEffect {
	pub position: Point3<f32>,
	pub sound: Sound,
	pub pitch: f32,
	pub volume: f32
}

#[repr(C)]
#[derive(Clone, PartialEq)]
pub struct WorldObject {
	pub zone: Point2<i32>,
	pub id: i32,
	pub unknown_a: i32,
	pub type_: WorldObjectType,
	//pad4
	pub position: Point3<i64>,
	pub orientation: i8,
	//pad3
	pub size: Hitbox,
	pub is_closed: bool,
	//pad3
	pub transform_time: i32,
	pub unknown_b: i32,
	//pad4
	pub interactor: i64
}

#[derive(Clone, PartialEq, Eq)]
pub struct P48 {
	pub zone: Point2<i32>,
	pub sub_packets: Vec<P48sub>
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct Pickup {
	pub interactor: CreatureId,
	pub item: Item
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct Kill {
	pub killer: CreatureId,
	pub victim: CreatureId,
	pub unknown: i32,
	pub xp: i32
}

#[repr(C)]
#[derive(Clone, PartialEq)]
pub struct Attack {
	pub target: i64,
	pub attacker: i64,
	pub damage: f32,
	//pad4
}

#[repr(C)]
#[derive(Clone, PartialEq, Eq)]
pub struct Mission {
	pub sector: Point2<i32>,
	pub unknown_a: i32,
	pub unknown_b: i32,
	pub unknown_c: i32,
	pub id: i32,
	pub kind: i32,
	pub boss: Race,
	pub level: i32,
	pub unknown_d: u8,
	pub state: MissionState,
	//pad2
	pub health_current: i32,
	pub health_maximum: i32,
	pub zone: Point2<i32>
}

bulk_impl!(CwSerializable for
	WorldEdit,
	//Hit
	Particle,
	SoundEffect,
	//Projectile
	WorldObject,
	//Drop
	//P48
	Pickup,
	Kill,
	Attack,
	//StatusEffect
	Mission
);