use async_compression::Level;
use async_compression::tokio::bufread::ZlibDecoder;
use async_compression::tokio::write::ZlibEncoder;
use nalgebra::{Point2, Point3, Vector3};
use rgb::{RGB, RGBA};
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::{ReadCwData, WriteCwData};
use crate::packet::{Hit, Projectile, StatusEffect, WorldUpdate};
use crate::packet::common::{CreatureId, Hitbox, Item, Race};
use crate::packet::world_update::drops::Drop;
use crate::utils::io_extensions::{ReadArbitrary, WriteArbitrary};

use self::mission::*;
use self::p48::*;

pub mod block;
pub mod particle;
pub mod sound;
pub mod world_object;
pub mod drops;
pub mod p48;
pub mod mission;

impl<Readable: AsyncRead + Unpin> ReadCwData<WorldUpdate> for Readable {
	async fn read_cw_data(&mut self) -> io::Result<WorldUpdate> {
		//todo: deduplicate (creature_update)
		let mut buffer = vec![0u8; self.read_arbitrary::<i32>().await? as usize];
		self.read_exact(&mut buffer).await?;

		let mut decoder = ZlibDecoder::new(buffer.as_slice());

		//todo: copypasta
		Ok(WorldUpdate {
			//explicit type annotation as a workaround for https://github.com/rust-lang/rust/issues/108362
			blocks        : ReadCwData::<Vec<Block                   >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			hits          : ReadCwData::<Vec<Hit                     >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			particles     : ReadCwData::<Vec<Particle                >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			sounds        : ReadCwData::<Vec<Sound                   >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			projectiles   : ReadCwData::<Vec<Projectile              >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			world_objects : ReadCwData::<Vec<WorldObject             >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			drops         : ReadCwData::<Vec<(Point2<i32>, Vec<Drop>)>>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			p48s          : ReadCwData::<Vec<P48                     >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			pickups       : ReadCwData::<Vec<Pickup                  >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			kills         : ReadCwData::<Vec<Kill                    >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			attacks       : ReadCwData::<Vec<Attack                  >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			status_effects: ReadCwData::<Vec<StatusEffect            >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?,
			missions      : ReadCwData::<Vec<Mission                 >>::read_cw_data(&mut decoder).await?,//decoder.read_cw_struct().await?
		})
	}
}

impl<Writable: AsyncWrite + Unpin> WriteCwData<WorldUpdate> for Writable {
	async fn write_cw_data(&mut self, world_update: &WorldUpdate) -> io::Result<()> {
		let mut buffer = vec![];

		let mut encoder = ZlibEncoder::with_quality(&mut buffer, Level::Best);

		//todo: copypasta
		encoder.write_cw_data(&world_update.blocks        ).await?;
		encoder.write_cw_data(&world_update.hits          ).await?;
		encoder.write_cw_data(&world_update.particles     ).await?;
		encoder.write_cw_data(&world_update.sounds        ).await?;
		encoder.write_cw_data(&world_update.projectiles   ).await?;
		encoder.write_cw_data(&world_update.world_objects ).await?;
		encoder.write_cw_data(&world_update.drops         ).await?;
		encoder.write_cw_data(&world_update.p48s          ).await?;
		encoder.write_cw_data(&world_update.pickups       ).await?;
		encoder.write_cw_data(&world_update.kills         ).await?;
		encoder.write_cw_data(&world_update.attacks       ).await?;
		encoder.write_cw_data(&world_update.status_effects).await?;
		encoder.write_cw_data(&world_update.missions      ).await?;

		encoder.shutdown().await?;

		self.write_arbitrary(&(buffer.len() as i32)).await?;
		self.write_all(&buffer).await
	}
}

#[repr(C)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block {
	pub position: Point3<i32>,
	pub color: RGB<u8>,
	pub kind: block::Kind,
	pub padding: i32 //todo: definitely NOT padding
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Particle {
	pub position: Point3<i64>,
	pub velocity: Vector3<f32>,
	pub color: RGBA<f32>,
	pub size: f32,
	pub count: i32,
	pub kind: particle::Kind,
	pub spread: f32,
	//pad4 //i32 according to cuwo
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Sound {
	pub position: Point3<f32>,
	pub kind: sound::Kind,
	pub pitch: f32,
	pub volume: f32
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct WorldObject {
	pub zone: Point2<i32>,
	pub id: i32,
	pub unknown_a: i32,
	pub kind: world_object::Kind,
	//pad4
	pub position: Point3<i64>,
	pub orientation: i8,//i32 according to cuwo
	//pad3
	pub size: Hitbox,
	pub is_closed: bool,
	//pad3
	pub transform_time: i32,
	pub unknown_b: i32,
	//pad4 //cuwo says 64bit padding??
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
	pub experience: i32
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
	pub unknown_a: i32,//always 0?
	pub unknown_b: i32,//always 0?
	pub unknown_c: i32,//always 0?
	pub id: i32,//doesnt matter at all?
	pub objective: Objective,
	pub race: Race,
	pub level: i32,
	pub rarity: u8,
	pub state: MissionState,
	//pad2
	pub progress_current: i32,
	pub progress_maximum: i32,
	pub zone: Point2<i32>//only matters for kind 1
}

impl<Readable: AsyncRead + Unpin> ReadCwData<Block      > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<Particle   > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<Sound      > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<WorldObject> for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<Pickup     > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<Kill       > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<Attack     > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<Mission    > for Readable {}

impl<Writable: AsyncWrite + Unpin> WriteCwData<Block      > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<Particle   > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<Sound      > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<WorldObject> for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<Pickup     > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<Kill       > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<Attack     > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<Mission    > for Writable {}
//Hit
//Projectile
//Drop
//P48
//StatusEffect


//todo: copypasta
impl From<Block> for WorldUpdate {
	fn from(value: Block) -> Self {
		Self {
			blocks: vec![value],
			..Default::default()
		}
	}
}

impl From<Hit> for WorldUpdate {
	fn from(value: Hit) -> Self {
		Self {
			hits: vec![value],
			..Default::default()
		}
	}
}

impl From<Particle> for WorldUpdate {
	fn from(value: Particle) -> Self {
		Self {
			particles: vec![value],
			..Default::default()
		}
	}
}

impl From<Sound> for WorldUpdate {
	fn from(value: Sound) -> Self {
		Self {
			sounds: vec![value],
			..Default::default()
		}
	}
}

impl From<Projectile> for WorldUpdate {
	fn from(value: Projectile) -> Self {
		Self {
			projectiles: vec![value],
			..Default::default()
		}
	}
}

impl From<WorldObject> for WorldUpdate {
	fn from(value: WorldObject) -> Self {
		Self {
			world_objects: vec![value],
			..Default::default()
		}
	}
}

impl From<(Point2<i32>, Vec<drops::Drop>)> for WorldUpdate {
	fn from(value: (Point2<i32>, Vec<drops::Drop>)) -> Self {
		Self {
			drops: vec![value],
			..Default::default()
		}
	}
}

impl From<P48> for WorldUpdate {
	fn from(value: P48) -> Self {
		Self {
			p48s: vec![value],
			..Default::default()
		}
	}
}

impl From<Pickup> for WorldUpdate {
	fn from(value: Pickup) -> Self {
		Self {
			pickups: vec![value],
			..Default::default()
		}
	}
}

impl From<Kill> for WorldUpdate {
	fn from(value: Kill) -> Self {
		Self {
			kills: vec![value],
			..Default::default()
		}
	}
}

impl From<Attack> for WorldUpdate {
	fn from(value: Attack) -> Self {
		Self {
			attacks: vec![value],
			..Default::default()
		}
	}
}

impl From<StatusEffect> for WorldUpdate {
	fn from(value: StatusEffect) -> Self {
		Self {
			status_effects: vec![value],
			..Default::default()
		}
	}
}

impl From<Mission> for WorldUpdate {
	fn from(value: Mission) -> Self {
		Self {
			missions: vec![value],
			..Default::default()
		}
	}
}