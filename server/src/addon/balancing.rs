use std::ops::Sub as _;
use std::sync::Arc;
use std::collections::HashMap;
use std::ptr;
use std::time::{Duration, Instant};

use config::{Config, ConfigError};
use serde::Deserialize;
use tokio::sync::RwLock;

use protocol::packet::{Hit, StatusEffect, WorldUpdate};
use protocol::rgb::RGBA;
use protocol::utils::constants::combat_classes::WATER_MAGE;
use protocol::packet::world_update::{particle, sound, Particle};
use protocol::packet::creature_update::equipment::Slot;
use protocol::packet::common::item::{Kind, Material};
use protocol::packet::common::CreatureId;
use protocol::packet::common::item::Kind::*;
use protocol::packet::common::item::kind::Weapon::*;
use protocol::packet::creature_update::CreatureFlag::{Climbing, Gliding};
use protocol::packet::creature_update::equipment::Slot::RightWeapon;
use protocol::packet::creature_update::Occupation::Rogue;
use protocol::packet::creature_update::PhysicsFlag::{OnGround, Swimming};
use protocol::packet::hit;
use protocol::packet::status_effect::Kind::{Anger, Swiftness};
use protocol::packet::world_update::Sound;
use protocol::packet::world_update::sound::Kind::{Magic01, SpikeTrap};

use crate::server::creature::Creature;
use crate::server::player::Player;
use crate::server::Server;

#[derive(Debug)]
pub struct Balancing {
	values: BalanceConfigValues,
	airtime_map: RwLock<HashMap<CreatureId, (Instant, bool)>>//todo: figure out a proper name
}

impl Balancing {
	pub fn new(config: &Config) -> Result<Self, ConfigError> {
		Ok(Self {
			values: config.get("balance")?,
			airtime_map: Default::default()
		})
	}

	pub async fn on_hit(&self, server: &Server, hit: &mut Hit, source: &Creature, target: &Arc<Player>) {
		let target_creature = target.character.read().await;
		self.ignite(server, hit, source, Arc::clone(target)).await;
		self.adjust_hit(hit, source, &target_creature);
	}

	pub async fn track_airtime(&self, source: &Player) {
		let character = source.character.read().await;

		if character.occupation != Rogue {
			return;
		}

		let flags_physics = character.flags_physics.clone();
		let flags = character.flags.clone();
		let position = character.position;
		drop(character); //otherwise we might hold this lock over multiple awaits

		let mut airtime_map = self.airtime_map.write().await;
		if flags_physics.get(OnGround) ||
			flags_physics.get(Swimming) ||
			flags.get(Gliding) ||
			flags.get(Climbing)
		{
			airtime_map.remove(&source.id);
			return;
		}

		let Some((timestamp, warned)) = airtime_map.get_mut(&source.id)
			else {
				airtime_map.insert(source.id, (Instant::now(), false));
				return;
			};

		let airtime = timestamp.elapsed();

		if airtime > Duration::from_secs(3) && !*warned {
			//todo: default
			let anger = StatusEffect {
				source: CreatureId(0),
				target: source.id,
				kind: Anger,
				modifier: 0.0,
				duration: 2000,
				creature_id3: source.id,//todo: is this needed?
			};
			let world_update = WorldUpdate {
				status_effects: vec![anger],
				sounds: vec![Sound::at(position, Magic01)],
				..Default::default()
			};
			source.send_ignoring(&world_update).await;

			*warned = true;
			return;
		}

		if airtime > Duration::from_secs(5) {
			let stun = Hit {
				target: source.id,
				stuntime: 3000,
				..Default::default()
			};
			let world_update = WorldUpdate {
				hits: vec![stun],
				sounds: vec![Sound::at(position, SpikeTrap)],
				..Default::default()
			};
			source.send_ignoring(&world_update).await;
			source.notify("you have been punished for glitching in the air too long").await;

			airtime_map.remove(&source.id);
		}
	}

	pub fn adjust_hit(&self, hit: &mut Hit, source: &Creature, target: &Creature) {
		let weapon_offense_multiplier = match source.equipment[RightWeapon].kind {
			Weapon(weapon)  => *self.values.damage.get(&weapon.to_string().to_lowercase()).unwrap_or(&1.0),
			_               => 1.0
		};

		let weapon_stun_bonus = match source.equipment[RightWeapon].kind {
			Weapon(weapon)  => *self.values.stun.get(&weapon.to_string().to_lowercase()).unwrap_or(&0),
			_               => 0
		};

		let class_offense_multiplier = *self
			.values
			.damage
			.get(source.combat_class().config_name())
			.unwrap_or(&1.0);

		let class_stun_bonus = *self
			.values
			.stun
			.get(source.combat_class().config_name())
			.unwrap_or(&0);

		let effective_damage_multiplier =
			self.values.damage["global"]
				* weapon_offense_multiplier
				* class_offense_multiplier;

		let effective_stun_bonus =
			self.values.stun["global"]
				+ weapon_stun_bonus
				+ class_stun_bonus;

		hit.damage *= effective_damage_multiplier;
		if hit.stuntime > 0 {
			hit.stuntime += effective_stun_bonus;
		}

		if hit.damage.is_sign_negative() {//heal
			let self_inflicted = ptr::eq(source, target);

			let heal_multiplier =
				if self_inflicted {
					if source.combat_class() == WATER_MAGE { self.values.heal_self }
					else                                   { *self.values.damage.get("unholy").unwrap_or(&1.0) }
					.sub(1.0) //self-heals are applied client side as well (bug), so we need to subtract the vanilla amount
				}
				else {
					self.values.heal_other
				};
			hit.damage *= heal_multiplier;
		} else {//hurt
			let equipment_defense_multiplier = 1.0 -
			target.equipment
				.iter()
				.map(|item| match item.kind {
					Weapon(Shield) => self.values.shield_defense,
					_              => 0.0
				})
				.sum::<f32>();

			hit.damage *= equipment_defense_multiplier;
		}
	}

	#[expect(clippy::cast_sign_loss, reason = "checked")]
	async fn ignite(&self, server: &Server, hit: &Hit, source: &Creature, target: Arc<Player>) {
		if source.combo % self.values.ignite_combo != 0 {
			return;
		}

		if hit.damage.is_sign_negative() || hit.kind != hit::Kind::Normal {
			return;
		}

		let fire_spirit_count = [
			Slot::LeftWeapon,
			Slot::RightWeapon
		].map(|slot| {
			let item = &source.equipment[slot];
			if item.kind == Kind::Void {
				return 0;
			}

			item.spirits
				.iter()
				.take(item.spirit_counter as _)
				.filter(|spirit| spirit.material == Material::Fire)
				.count()
		})
		.iter()
		.sum::<usize>();

		if fire_spirit_count == 0 {
			return;
		}

		let n_particles = hit.damage as usize / 25;

		let particles = [
			RGBA::new(1.0, 0.0, 0.0, 1.0),
			RGBA::new(1.0, 0.5, 0.0, 1.0),
			RGBA::new(1.0, 1.0, 0.0, 1.0)
		]
			.into_iter()
			.enumerate()
			.map(|(color_index, color)| Particle {
				position: hit.position,
				velocity: [0.0, 0.0, 0.0].into(),
				color,
				size: 0.1,
				count: (n_particles + color_index) as i32 / 3,
				kind: particle::Kind::NoGravity,
				spread: 1.0,
			})
			.collect();

		let damage = hit.damage
			* fire_spirit_count as f32
			* self.values.damage["fire_spirit"]
			/ 10.0;

		server.apply_dot(source, target, 10, 200, damage, sound::Kind::FireHit, Some(particles)).await;
	}

	pub fn adjust_manashield(&self, packet: &mut StatusEffect) {
		packet.duration = self.values.manashield_duration;
		if let Some(absolute_value) = self.values.manashield_capacity_absolute {
			packet.modifier = absolute_value;
		} else {
			packet.modifier *= self.values.manashield_capacity_relative;
		}
	}
}

pub async fn buff_warfrenzy(warfrenzy: &StatusEffect, server: &Server) {
	let swiftness = StatusEffect {
		kind: Swiftness,
		..*warfrenzy
	};
	// sending this separately from the original status effect
	// as that one isn't sent back to the source
	server.broadcast(&WorldUpdate::from(swiftness), None).await;
}

pub fn adjust_blocking(hit: &mut Hit, attacker_creature: &Creature) {
	if hit.kind != hit::Kind::Block {
		return
	}

	let has_shield = attacker_creature
		.equipment
		.iter()
		.any(|item| item.kind == Weapon(Shield));
	hit.damage *= if has_shield { 0.5 } else { 0.0 };
}

#[derive(Debug, Deserialize)]
struct BalanceConfigValues {
	heal_self: f32,
	heal_other: f32,
	shield_defense: f32,
	damage: HashMap<String, f32>,
	stun: HashMap<String, i32>,
	manashield_duration: i32,
	manashield_capacity_relative: f32,
	manashield_capacity_absolute: Option<f32>,
	ignite_combo: i32
}