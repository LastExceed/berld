use std::io::{Error, Read, Write};

use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};
use crate::packet::hit::Hit;
use crate::packet::projectile::Projectile;
use crate::packet::status_effect::StatusEffect;
use crate::utils::{ReadExtension, WriteExtension};

use self::attack::Attack;
use self::chunk_loot::ChunkLoot;
use self::kill::Kill;
use self::mission::Mission;
use self::p48::P48;
use self::particle::Particle;
use self::pickup::Pickup;
use self::sound_effect::SoundEffect;
use self::world_edit::WorldEdit;
use self::world_object::WorldObject;

pub mod world_edit;
pub mod particle;
pub mod sound_effect;
pub mod world_object;
pub mod chunk_loot;
pub mod p48;
pub mod pickup;
pub mod kill;
pub mod attack;
pub mod mission;

#[derive(Default)]
pub struct WorldUpdate {
	pub world_edits: Vec<WorldEdit>,
	pub hits: Vec<Hit>,
	pub particles: Vec<Particle>,
	pub sound_effects: Vec<SoundEffect>,
	pub projectiles: Vec<Projectile>,
	pub world_objects: Vec<WorldObject>,
	pub chunk_loots: Vec<ChunkLoot>,
	pub p48s: Vec<P48>,
	pub pickups: Vec<Pickup>,
	pub kills: Vec<Kill>,
	pub attacks: Vec<Attack>,
	pub status_effects: Vec<StatusEffect>,
	pub missions: Vec<Mission>
}

impl CwSerializable for WorldUpdate {
	fn read_from(reader: &mut impl Read) -> Result<Self, Error> {
		//todo: deduplicate (creature_update)
		let mut buffer = vec![0u8; reader.read_struct::<i32>()? as usize];
		reader.read_exact(&mut buffer)?;

		let mut decoder = ZlibDecoder::new(buffer.as_slice());

		//todo: copypasta
		Ok(Self {
			world_edits   : Vec::read_from(&mut decoder)?,
			hits          : Vec::read_from(&mut decoder)?,
			particles     : Vec::read_from(&mut decoder)?,
			sound_effects : Vec::read_from(&mut decoder)?,
			projectiles   : Vec::read_from(&mut decoder)?,
			world_objects : Vec::read_from(&mut decoder)?,
			chunk_loots   : Vec::read_from(&mut decoder)?,
			p48s          : Vec::read_from(&mut decoder)?,
			pickups       : Vec::read_from(&mut decoder)?,
			kills         : Vec::read_from(&mut decoder)?,
			attacks       : Vec::read_from(&mut decoder)?,
			status_effects: Vec::read_from(&mut decoder)?,
			missions      : Vec::read_from(&mut decoder)?
		})
	}

	fn write_to(&self, writer: &mut impl Write) -> Result<(), Error> {
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
			self.chunk_loots   .write_to(&mut encoder)?;
			self.p48s          .write_to(&mut encoder)?;
			self.pickups       .write_to(&mut encoder)?;
			self.kills         .write_to(&mut encoder)?;
			self.attacks       .write_to(&mut encoder)?;
			self.status_effects.write_to(&mut encoder)?;
			self.missions      .write_to(&mut encoder)?;

			encoder.flush()?;
		}
		writer.write_struct(&(buffer.len() as i32))?;
		writer.write_all(&buffer)
	}
}
impl Packet for WorldUpdate {
	const ID: PacketId = PacketId::WorldUpdate;
}
impl PacketFromServer for WorldUpdate {}