use async_compression::Level;
use async_compression::tokio::bufread::ZlibDecoder;
use async_compression::tokio::write::ZlibEncoder;
use async_trait::async_trait;
use nalgebra::{Point2, Point3, Vector3};
use rgb::{RGB, RGBA};
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

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

#[async_trait]
impl CwSerializable for WorldUpdate {
	async fn read_from<Readable: AsyncRead + Unpin + Send>(readable: &mut Readable) -> io::Result<Self> {
		//todo: deduplicate (creature_update)
		let mut buffer = vec![0u8; readable.read_struct::<i32>().await? as usize];
		readable.read_exact(&mut buffer).await?;

		let mut decoder = ZlibDecoder::new(buffer.as_slice());

		//todo: copypasta
		Ok(Self {
			world_edits   : Vec::read_from(&mut decoder).await?,
			hits          : Vec::read_from(&mut decoder).await?,
			particles     : Vec::read_from(&mut decoder).await?,
			sound_effects : Vec::read_from(&mut decoder).await?,
			projectiles   : Vec::read_from(&mut decoder).await?,
			world_objects : Vec::read_from(&mut decoder).await?,
			drops         : Vec::read_from(&mut decoder).await?,
			p48s          : Vec::read_from(&mut decoder).await?,
			pickups       : Vec::read_from(&mut decoder).await?,
			kills         : Vec::read_from(&mut decoder).await?,
			attacks       : Vec::read_from(&mut decoder).await?,
			status_effects: Vec::read_from(&mut decoder).await?,
			missions      : Vec::read_from(&mut decoder).await?
		})
	}

	async fn write_to<Writable: AsyncWrite + Unpin + Send>(&self, writable: &mut Writable) -> io::Result<()> {
		let mut buffer = vec![];
		{
			let mut encoder = ZlibEncoder::with_quality(&mut buffer, Level::Best);

			//todo: copypasta
			self.world_edits   .write_to(&mut encoder).await?;
			self.hits          .write_to(&mut encoder).await?;
			self.particles     .write_to(&mut encoder).await?;
			self.sound_effects .write_to(&mut encoder).await?;
			self.projectiles   .write_to(&mut encoder).await?;
			self.world_objects .write_to(&mut encoder).await?;
			self.drops         .write_to(&mut encoder).await?;
			self.p48s          .write_to(&mut encoder).await?;
			self.pickups       .write_to(&mut encoder).await?;
			self.kills         .write_to(&mut encoder).await?;
			self.attacks       .write_to(&mut encoder).await?;
			self.status_effects.write_to(&mut encoder).await?;
			self.missions      .write_to(&mut encoder).await?;

			encoder.shutdown().await?;
		}
		writable.write_struct(&(buffer.len() as i32)).await?;
		writable.write_all(&buffer).await
	}
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct WorldEdit {
	pub position: Point3<i32>,
	pub color: RGB<u8>,
	pub block_type: BlockType,
	pub padding: i32
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
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
#[derive(Debug, PartialEq, Clone)]
pub struct SoundEffect {
	pub position: Point3<f32>,
	pub sound: Sound,
	pub pitch: f32,
	pub volume: f32
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct P48 {
	pub zone: Point2<i32>,
	pub sub_packets: Vec<P48sub>
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Pickup {
	pub interactor: CreatureId,
	pub item: Item
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Kill {
	pub killer: CreatureId,
	pub victim: CreatureId,
	pub unknown: i32,
	pub xp: i32
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Attack {
	pub target: i64,
	pub attacker: i64,
	pub damage: f32,
	//pad4
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone)]
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