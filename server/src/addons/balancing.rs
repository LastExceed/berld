use std::ptr;

use protocol::packet::{Hit, StatusEffect, WorldUpdate};
use protocol::packet::common::item::Kind::*;
use protocol::packet::common::item::kind::Weapon::*;
use protocol::packet::creature_update::equipment::Slot::RightWeapon;
use protocol::packet::status_effect::StatusEffectType::Swiftness;
use protocol::utils::constants::combat_classes::*;

use crate::creature::Creature;
use crate::server::Server;

pub async fn buff_warfrenzy(warfrenzy: &StatusEffect, server: &Server) {
	let swiftness = StatusEffect {
		type_: Swiftness,
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
	hit.stuntime += effective_stun_bonus;
}