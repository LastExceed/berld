use std::io::{Error, Read, Write};
use flate2::Compression;
use flate2::read::{ZlibDecoder};
use flate2::write::ZlibEncoder;
use crate::packet::{CwSerializable, Packet, PacketFromServer, PacketId};
use crate::packet::hit::Hit;
use crate::packet::projectile::Projectile;
use crate::packet::status_effect::StatusEffect;
use crate::utils::{ReadExtension, WriteExtension};
use self::attack::Attack;
use self::kill::Kill;
use self::mission::Mission;
use self::p48::P48;
use self::pickup::Pickup;
use self::chunk_loot::ChunkLoot;
use self::particle::Particle;
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
	fn read_from<T: Read>(reader: &mut T) -> Result<Self, Error> {
		//todo: deduplicate (creature_update)
		let size = reader.read_struct::<i32>()?;
		let mut buffer = vec![0u8; size as usize];
		reader.read_exact(&mut buffer)?;

		let mut decoder = Box::new(ZlibDecoder::new(buffer.as_slice())) as Box<dyn Read>;

		//todo: copypasta
		let world_edit_count = decoder.read_struct::<i32>()?;
		let mut world_edits = Vec::with_capacity(world_edit_count as usize);
		for _ in 0..world_edit_count {
			world_edits.push(WorldEdit::read_from(&mut decoder)?);
		}
		let hit_count = decoder.read_struct::<i32>()?;
		let mut hits = Vec::with_capacity(hit_count as usize);
		for _ in 0..hit_count {
			hits.push(Hit::read_from(&mut decoder)?);
		}
		let particle_count = decoder.read_struct::<i32>()?;
		let mut particles = Vec::with_capacity(particle_count as usize);
		for _ in 0..particle_count {
			particles.push(Particle::read_from(&mut decoder)?);
		}
		let sound_effect_count = decoder.read_struct::<i32>()?;
		let mut sound_effects = Vec::with_capacity(sound_effect_count as usize);
		for _ in 0..sound_effect_count {
			sound_effects.push(SoundEffect::read_from(&mut decoder)?);
		}
		let projectile_count = decoder.read_struct::<i32>()?;
		let mut projectiles = Vec::with_capacity(projectile_count as usize);
		for _ in 0..projectile_count {
			projectiles.push(Projectile::read_from(&mut decoder)?);
		}
		let world_object_count = decoder.read_struct::<i32>()?;
		let mut world_objects = Vec::with_capacity(world_object_count as usize);
		for _ in 0..world_object_count {
			world_objects.push(WorldObject::read_from(&mut decoder)?);
		}
		let chunk_loot_count = decoder.read_struct::<i32>()?;
		let mut chunk_loots = Vec::with_capacity(chunk_loot_count as usize);
		for _ in 0..chunk_loot_count {
			chunk_loots.push(ChunkLoot::read_from(&mut decoder)?);
		}
		let p48_count = decoder.read_struct::<i32>()?;
		let mut p48s = Vec::with_capacity(p48_count as usize);
		for _ in 0..p48_count {
			p48s.push(P48::read_from(&mut decoder)?);
		}
		let pickup_count = decoder.read_struct::<i32>()?;
		let mut pickups = Vec::with_capacity(pickup_count as usize);
		for _ in 0..pickup_count {
			pickups.push(Pickup::read_from(&mut decoder)?);
		}
		let kill_count = decoder.read_struct::<i32>()?;
		let mut kills = Vec::with_capacity(kill_count as usize);
		for _ in 0..kill_count {
			kills.push(Kill::read_from(&mut decoder)?);
		}
		let attack_count = decoder.read_struct::<i32>()?;
		let mut attacks = Vec::with_capacity(attack_count as usize);
		for _ in 0..attack_count {
			attacks.push(Attack::read_from(&mut decoder)?);
		}
		let status_effect_count = decoder.read_struct::<i32>()?;
		let mut status_effects = Vec::with_capacity(status_effect_count as usize);
		for _ in 0..status_effect_count {
			status_effects.push(StatusEffect::read_from(&mut decoder)?);
		}
		let mission_count = decoder.read_struct::<i32>()?;
		let mut missions = Vec::with_capacity(mission_count as usize);
		for _ in 0..mission_count {
			missions.push(Mission::read_from(&mut decoder)?);
		}

		let instance = Self {
			world_edits,
			hits,
			particles,
			sound_effects,
			projectiles,
			world_objects,
			chunk_loots,
			p48s,
			pickups,
			kills,
			attacks,
			status_effects,
			missions
		};

		Ok(instance)
	}

	fn write_to<T: Write>(&self, writer: &mut T) -> Result<(), Error> {
		let mut buffer = Vec::new();//todo: required capacity can be computed in advance
		{
			let mut encoder = Box::new(ZlibEncoder::new(&mut buffer, Compression::default())) as Box<dyn Write>;

			//todo: copypasta
			encoder.write_struct(&(self.world_edits.len() as i32))?;
			for it in &self.world_edits {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.hits.len() as i32))?;
			for it in &self.hits {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.particles.len() as i32))?;
			for it in &self.particles {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.sound_effects.len() as i32))?;
			for it in &self.sound_effects {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.projectiles.len() as i32))?;
			for it in &self.projectiles {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.world_objects.len() as i32))?;
			for it in &self.world_objects {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.chunk_loots.len() as i32))?;
			for it in &self.chunk_loots {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.p48s.len() as i32))?;
			for it in &self.p48s {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.pickups.len() as i32))?;
			for it in &self.pickups {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.kills.len() as i32))?;
			for it in &self.kills {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.attacks.len() as i32))?;
			for it in &self.attacks {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.status_effects.len() as i32))?;
			for it in &self.status_effects {
				it.write_to(&mut encoder)?;
			}
			encoder.write_struct(&(self.missions.len() as i32))?;
			for it in &self.missions {
				it.write_to(&mut encoder)?;
			}

			encoder.flush()?;
		}

		writer.write_struct(&(buffer.len() as i32))?;
		writer.write_all(&buffer)
	}
}
impl Packet for WorldUpdate {
	fn id() -> PacketId {
		PacketId::WorldUpdate
	}
}
impl PacketFromServer for WorldUpdate {}