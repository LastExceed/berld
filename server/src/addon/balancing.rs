use std::collections::HashMap;
use std::ptr;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use protocol::packet::{Hit, StatusEffect, WorldUpdate};
use protocol::packet::common::CreatureId;
use protocol::packet::common::item::Kind::*;
use protocol::packet::common::item::kind::Weapon::*;
use protocol::packet::creature_update::CreatureFlag::{Climbing, Gliding};
use protocol::packet::creature_update::equipment::Slot::RightWeapon;
use protocol::packet::creature_update::Occupation::Rogue;
use protocol::packet::creature_update::PhysicsFlag::{OnGround, Swimming};
use protocol::packet::status_effect::Kind::{Anger, Swiftness};
use protocol::packet::world_update::Sound;
use protocol::packet::world_update::sound::Kind::{Magic01, SpikeTrap};
use protocol::utils::constants::combat_classes::*;

use crate::server::creature::Creature;
use crate::server::player::Player;
use crate::server::Server;

pub struct AirTimeTracker {
	airtime_map: RwLock<HashMap<CreatureId, (Instant, bool)>>//todo: figure out a proper name
}

impl AirTimeTracker {
	pub fn new() -> Self {
		Self {
			airtime_map: RwLock::new(HashMap::new())
		}
	}

	pub async fn on_creature_update(&self, source: &Player) {
		let character = source.character.read().await;

		if character.occupation != Rogue {
			return;
		}

		let mut airtime_map = self.airtime_map.write().await;
		if character.flags_physics.get(OnGround) ||
			character.flags_physics.get(Swimming) ||
			character.flags.get(Gliding) ||
			character.flags.get(Climbing)
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
			let sound = Sound::at(
				character.position,
				Magic01
			);
			let world_update = WorldUpdate {
				status_effects: vec![anger],
				sounds: vec![sound],
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
			let sound = Sound::at(
				character.position,
				SpikeTrap
			);
			let world_update = WorldUpdate {
				hits: vec![stun],
				sounds: vec![sound],
				..Default::default()
			};
			source.send_ignoring(&world_update).await;
			source.notify("you have been punished for glitching in the air too long").await;

			airtime_map.remove(&source.id);
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

const GLOBAL_DAMAGE_MULTIPLIER: f32 = 0.5;
const GLOBAL_STUN_BONUS: i32 = -375;

pub fn adjust_hit(hit: &mut Hit, source: &Creature, target: &Creature) {
	let heals = hit.damage.is_sign_negative();

	if heals {
		let self_inflicted = ptr::eq(source, target);

		let heal_multiplier =
			if self_inflicted { 0.5 - 1.0 } //self-heals are applied client side as well (bug), so we need to subtract the vanilla amount
			else              { 0.3 };
		hit.damage *= heal_multiplier;
		return;
	}

	let (weapon_offense_multiplier, weapon_stun_bonus) =
		match source.equipment[RightWeapon].kind {
			Weapon(Wand)  => (1.0, 375),
			Weapon(Staff) => (1.1,   0),
			Weapon(Fist)  => (1.2, 375),
			_             => (1.0,   0)
		};

	let (class_offense_multiplier, class_stun_bonus) =
		match source.combat_class() {
			BERSERKER => (1.2, 750),
			SNIPER    => (1.1, 375),
			SCOUT     => (1.1,   0),
			ASSASSIN  => (1.5,   0),
			NINJA     => (0.9,   0),
			FIRE_MAGE => (0.8,   0),
			_         => (1.0,   0)
		};

	let equipment_defense_multiplier = 1.0 +
		target.equipment
			.iter()
			.map(|item| match item.kind {
				Weapon(Shield) => 0.25,
				_              => 0.0
			})
			.sum::<f32>();

	let effective_damage_multiplier =
		GLOBAL_DAMAGE_MULTIPLIER
			* weapon_offense_multiplier
			* class_offense_multiplier
			* equipment_defense_multiplier.recip();

	let effective_stun_bonus =
		GLOBAL_STUN_BONUS
			+ weapon_stun_bonus
			+ class_stun_bonus;

	hit.damage *= effective_damage_multiplier;
	if hit.stuntime > 0 {
		hit.stuntime += effective_stun_bonus;
	}
}