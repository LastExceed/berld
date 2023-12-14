#![expect(unused_variables, clippy::missing_const_for_fn, reason = "for consistency, should probably enforce this with a trait somehow")]

use std::default::Default;
use std::time::Duration;

use boolinator::Boolinator;
use strum::IntoEnumIterator;
use tap::Pipe;

use protocol::nalgebra::Point3;
use protocol::packet::common::{CreatureId, EulerAngles, Hitbox, item};
use protocol::packet::common::item::{Kind, KindDiscriminants};
use protocol::packet::common::Race::*;
use protocol::packet::creature_update::{Affiliation, Animation, CreatureFlag, PhysicsFlag};
use protocol::packet::creature_update::Animation::*;
use protocol::packet::creature_update::equipment::Slot;
use protocol::packet::creature_update::multipliers::Multiplier::*;
use protocol::packet::creature_update::Occupation::*;
use protocol::packet::creature_update::skill_tree::Skill;
use protocol::packet::creature_update::Specialization::*;
use protocol::utils::{maximum_experience_of, power_of};
use protocol::utils::constants::combat_classes::*;
use protocol::utils::constants::{PLAYABLE_RACES, TWO_HANDED_WEAPONS};
use protocol::utils::constants::rarity::*;
use protocol::utils::flagset::FlagSet;

use crate::addon::anti_cheat;
use crate::addon::anti_cheat::*;
use crate::addon::anti_cheat::creature_update::animation::animations_avilable_with;
use crate::addon::anti_cheat::creature_update::equipment::allowed_materials;
use crate::server::creature::Creature;

mod animation;
mod equipment;

pub(super) fn inspect_position(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_rotation(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	//usually 0, except
	//- rounding errors
	//- 60f..=0 when swimming (or shortly afterwards)
	//- 20f when teleporting
	updated_state.rotation.pitch
		.is_finite()
		.ok_or("rotation.yaw wasn't finite")?;
	updated_state.rotation.roll
		.ensure_within(&(-90.0..=90.0), "rotation.roll")?;
	updated_state.rotation.yaw//normally -180..=180, but over-/underflows while attacking
		.is_finite()
		.ok_or("rotation.yaw wasn't finite".into())
}

pub(super) fn inspect_velocity(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_acceleration(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	//todo: investigate false positives
//	let limit_xy = Vector3::<f32>::new(80.0, 80.0, 0.0).magnitude() + 0.00001; //113,1370849898476; //todo: would epsilon suffice?
//	let actual_xy = acceleration.xy().magnitude();
//	if !updated_state.flags.get(CreatureFlag::Gliding) {
//		actual_xy.ensure_within(&(0.0..=limit_xy), "acceleration.horizontal")?;
//	}

	#[expect(clippy::dbg_macro, reason = "testing in production lol")]
	if updated_state.flags_physics.get(PhysicsFlag::Swimming) {
		updated_state.acceleration.z.ensure_within(&(-80.0..=80.0), "acceleration.vertical")
	} else if updated_state.flags.get(CreatureFlag::Climbing) || previous_state.flags.get(CreatureFlag::Climbing) {//possible fix for a false positive
		updated_state.acceleration.z.ensure_one_of(&[-16.0, 0.0, 16.0], "acceleration.vertical")
	} else {
		updated_state.acceleration.z.ensure_exact(&0.0, "acceleration.vertical")
	}
}

pub(super) fn inspect_velocity_extra(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	let (max_xy, max_z): (f32, f32) =
		match updated_state.occupation {
			Ranger => (35.0, 17.0),
			_      => ( 0.1,  0.0)//0.1 because the game doesnt reset all the way to 0
		};

	updated_state.velocity_extra.xy()
		.magnitude()
		.ensure_at_most(max_xy, "velocity_extra.horizontal")?;
	updated_state.velocity_extra.z
		.ensure_within(&(0.0..=max_z), "velocity_extra.vertical")
}

pub(super) fn inspect_head_tilt(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.head_tilt
		.ensure_within(&(-32.5..=45.0), "head_tilt")//negative when attacking downwards
}

pub(super) fn inspect_flags_physics(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_affiliation(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.affiliation
		.ensure_exact(&Affiliation::Player, "affiliation")
}

pub(super) fn inspect_race(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.race
		.ensure_one_of(PLAYABLE_RACES.as_slice(), "race")
}

pub(super) fn inspect_animation(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	let allowed_animations = animations_avilable_with(updated_state.combat_class(), &updated_state.equipment);

	updated_state.animation
		.ensure_one_of(&allowed_animations, "animation")
}

pub(super) fn inspect_animation_time(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	const TIMELESS_ANIMATIONS: [Animation; 12] = [
		Idle,
		Stealth,
		Sail,
		Sit,
		PetFoodPresent,
		Sleep,
		//todo: separate?
		ShieldM2Charging,
		GreatweaponM2Charging,
		BoomerangM2Charging,
		CrossbowM2Charging,
		UnarmedM2Charging,
		BowM2Charging
		//todo: some unused animations are timeless as well
	];

	updated_state.animation_time
		.ensure_not_negative("animation time")?;

	if !updated_state.animation.present_in(&TIMELESS_ANIMATIONS) {
		updated_state.animation_time.ensure_at_most(10_000, "animation time")?;
	};

	// if updated_state.animation_time < former_state.animation_time && updated_state.animation == FireExplosionShort {
	// 	//todo: detect fire spam
	// }

	Ok(())
}

pub(super) fn inspect_combo(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.combo
		.ensure_not_negative("combo")
}

#[expect(clippy::cast_sign_loss, reason = "checked at runtime")]
pub(super) async fn inspect_combo_timeout(previous_state: &Creature, updated_state: &Creature, player: &Player) -> anti_cheat::Result {
	updated_state.combo_timeout
		.ensure_not_negative("combo_timeout")?;

	let ac_data = &mut player.addon_data.write().await.anti_cheat_data;

	let was_dead = previous_state.health == 0.0;
	let is_dead = updated_state.health == 0.0;

	if was_dead && is_dead {
		//clock freezes while dead
		return updated_state.combo_timeout.ensure_exact(&previous_state.combo_timeout, "combo_timeout");
	}

	let init = ac_data.last_combo_update.is_none();//todo: move to ac data init?
	let respawn = was_dead && !is_dead; //timeout resets to 0 on respawn
	let hit = updated_state.combo_timeout <= previous_state.combo_timeout; //equal incase of seed change lag

	if init || respawn || hit {
		ac_data.last_combo_update = Some(Instant::now() - Duration::from_millis(updated_state.combo_timeout as _));
		ac_data.strikes = 0;
		return Ok(());
	}

	let elapsed_nanos = ac_data.last_combo_update.unwrap().elapsed().as_nanos() as i128;
	let reportet_nanos = updated_state.combo_timeout as i128 * 1_000_000;

	let delta = reportet_nanos - elapsed_nanos;

	if delta.abs() > 500_000_000 {
		ac_data.strikes += 1;
		if ac_data.strikes >= 50 {
			if ac_data.last_lag_spike.is_some_and(|time| time.elapsed() < Duration::from_secs(15)) {
				return Err("timewarp".into())
			}
			let now = Instant::now();
			ac_data.last_lag_spike = Some(now);
			ac_data.last_combo_update = Some(now - Duration::from_millis(updated_state.combo_timeout as _));
			ac_data.strikes = 0;
		}
	} else if ac_data.strikes > 0 {
		ac_data.strikes -= 1;
	}

	//println!("{} {}", ac_data.strikes, delta as f64 / 1_000_000.0);

	Ok(())
}

#[expect(clippy::too_many_lines, reason = "TODO")] //TODO: extract constants
pub(super) fn inspect_appearance(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.appearance.flags.ensure_exact(&FlagSet::default(), "appearance.flags")?;

	updated_state.appearance.tail_model.ensure_exact(&-1, "appearance.tail_model")?;
	updated_state.appearance.shoulder2model.ensure_exact(&-1, "appearance.shoulder2model")?;
	updated_state.appearance.wing_model.ensure_exact(&-1, "appearance.wing_model")?;

	updated_state.appearance.hand_size.ensure_exact(&1.0, "appearance.hand_size")?;
	updated_state.appearance.foot_size.ensure_exact(&0.98, "appearance.footSize")?;
	updated_state.appearance.tail_size.ensure_exact(&0.8, "appearance.tailSize")?;
	updated_state.appearance.shoulder2size.ensure_exact(&1.0, "appearance.shoulder1Size")?;
	updated_state.appearance.wing_size.ensure_exact(&1.0, "appearance.wingSize")?;

	updated_state.appearance.body_offset.ensure_exact(&Point3::new(0.0, 0.0, -5.0), "appearance.bodyOffset")?;
	updated_state.appearance.head_offset.ensure_exact(
		&if updated_state.race == OrcFemale {
			Point3::new(0.0, 1.5, 4.0)
		} else {
			Point3::new(0.0, 0.5, 5.0)
		},
		"appearance.headOffset"
	)?;
	updated_state.appearance.hand_offset.ensure_exact(&Point3::new(6.0, 0.0,  0.0), "appearance.handOffset")?;
	updated_state.appearance.foot_offset.ensure_exact(&Point3::new(3.0, 1.0,-10.5), "appearance.footOffset")?;
	updated_state.appearance.tail_offset.ensure_exact(&Point3::new(0.0,-8.0,  2.0), "appearance.tailOffset")?;
	updated_state.appearance.wing_offset.ensure_exact(&Point3::new(0.0, 0.0,  0.0), "appearance.wingOffset")?;

	updated_state.appearance.body_rotation.ensure_exact(&0.0, "appearance.bodyRotation")?;
	updated_state.appearance.hand_rotation.ensure_exact(&EulerAngles::default(), "appearance.handRotation")?;
	updated_state.appearance.feet_rotation.ensure_exact(&0.0, "appearance.feetRotation")?;
	updated_state.appearance.wing_rotation.ensure_exact(&0.0, "appearance.wingRotation")?;
	updated_state.appearance.tail_rotation.ensure_exact(&0.0, "appearance.tail_rotation")?;

	//todo: move all this to protocol crate
	let hitbox_small = Hitbox {
		width: 0.80,
		depth: 0.80,
		height: 1.80
	};
	let hitbox_medium = Hitbox {
		width: 0.96000004,
		depth: 0.96000004,
		height: 2.16
	};
	let hitbox_large = Hitbox {
		width: 1.04,
		depth: 1.04,
		height: 2.34
	};

	let (
		allowed_creature_size,
		allowed_head_model,
		allowed_hair_model,
		allowed_hand_model,
		allowed_foot_model,
		allowed_body_model,
		allowed_head_size,
		allowed_body_size,
		allowed_shoulder1size,
		allowed_weapon_size
	) = match updated_state.race {
		ElfMale => (
			hitbox_medium,
			1236..=1239,
			1280..=1289,
			430..=430,
			432,
			1,
			1.01,
			1.00,
			1.00,
			0.95
		),
		ElfFemale => (
			hitbox_medium,
			1240..=1245,
			1290..=1299,
			430..=430,
			432,
			0,
			1.01,
			1.00,
			1.00,
			0.95
		),
		HumanMale => (
			hitbox_medium,
			1246..=1251,
			1252..=1266,
			430..=431,
			432,
			1,
			1.01,
			1.00,
			1.00,
			0.95
		),
		HumanFemale => (
			hitbox_medium,
			1267..=1272,
			1273..=1279,
			430..=431,
			432,
			1,
			1.01,
			1.00,
			1.00,
			0.95
		),
		GoblinMale => (
			hitbox_small,
			75..=79,
			80..=85,
			97..=97,
			432,
			0,
			1.01,
			1.00,
			1.00,
			1.20
		),
		GoblinFemale => (
			hitbox_small,
			86..=90,
			91..=96,
			97..=97,
			432,
			0,
			1.01,
			1.00,
			1.00,
			1.20
		),
		LizardmanMale => (
			hitbox_medium,
			98..=99,
			100..=105,
			111..=111,
			113,
			112,
			1.01,
			1.00,
			1.00,
			0.95
		),
		LizardmanFemale => (
			hitbox_medium,
			106..=111,
			100..=105,
			111..=111,
			113,
			112,
			1.01,
			1.00,
			1.00,
			0.95
		),
		DwarfMale => (
			hitbox_small,
			282..=286,
			287..=289,
			430..=431,
			432,
			300,
			0.90,
			1.00,
			1.00,
			1.20
		),
		DwarfFemale => (
			hitbox_small,
			290..=294,
			295..=299,
			430..=431,
			432,
			301,
			0.90,
			1.00,
			1.00,
			1.20
		),
		OrcMale => (
			hitbox_large,
			1300..=1304,
			1310..=1319,
			302..=302,
			432,
			0,
			0.90,
			1.00,
			1.20,
			0.95
		),
		OrcFemale => (
			hitbox_large,
			1305..=1309,
			1320..=1323,
			302..=302,
			432,
			0,
			0.80,
			0.95,
			1.10,
			0.95
		),
		FrogmanMale => (
			hitbox_medium,
			1324..=1328,
			1329..=1333,
			1342..=1342,
			432,
			1,
			1.01,
			1.00,
			1.00,
			0.95
		),
		FrogmanFemale => (
			hitbox_medium,
			1334..=1337,
			1338..=1341,
			1342..=1342,
			432,
			1,
			1.01,
			1.00,
			1.00,
			0.95
		),
		UndeadMale => (
			hitbox_medium,
			303..=308,
			309..=314,
			327..=327,
			432,
			0,
			0.90,
			1.00,
			1.00,
			0.95
		),
		UndeadFemale => (
			hitbox_medium,
			315..=320,
			321..=326,
			327..=327,
			432,
			0,
			0.90,
			1.00,
			1.00,
			0.95
		),
		_ => unreachable!("race has already been ensured to be one of the above at this point")
	};

	updated_state.appearance.creature_size.ensure_exact (&allowed_creature_size, "appearance.creature.Size")?;
	updated_state.appearance.head_model   .ensure_within(&allowed_head_model   , "appearance.headModel")?;
	updated_state.appearance.hair_model   .ensure_within(&allowed_hair_model   , "appearance.hairModel")?;
	updated_state.appearance.hand_model   .ensure_within(&allowed_hand_model   , "appearance.handModel")?;
	updated_state.appearance.foot_model   .ensure_exact (&allowed_foot_model   , "appearance.footModel")?;
	updated_state.appearance.body_model   .ensure_exact (&allowed_body_model   , "appearance.bodyModel")?;
	updated_state.appearance.head_size    .ensure_exact (&allowed_head_size    , "appearance.headSize")?;
	updated_state.appearance.body_size    .ensure_exact (&allowed_body_size    , "appearance.bodySize")?;
	updated_state.appearance.shoulder1size.ensure_exact (&allowed_shoulder1size, "appearance.shoulder2Size")?;
	updated_state.appearance.weapon_size  .ensure_exact (&allowed_weapon_size  , "appearance.weaponSize")

}

pub(super) fn inspect_flags(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.flags.get(CreatureFlag::FriendlyFire)
		.ensure_exact(&false, "flags[FriendlyFire]")?;

	if updated_state.combat_class() != SNIPER {
		updated_state.flags.get(CreatureFlag::Sniping)
			.ensure_exact(&false, "flags[Sniping]")?;
	}

	if updated_state.equipment[Slot::Lamp].kind == item::Kind::Void {
		updated_state.flags.get(CreatureFlag::Lamp)
			.ensure_exact(&false, "flags[Lamp]")?;
	}
	Ok(())
}

pub(super) fn inspect_effect_time_dodge(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.effect_time_dodge
		.ensure_within(&(0..=600), "effect_time_dodge")
}

pub(super) fn inspect_effect_time_stun(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	//todo: freeze at 0 would bypass this
	if updated_state.effect_time_stun > previous_state.effect_time_stun {
		if previous_state.health == 0.0 && updated_state.health > 0.0 {
			updated_state.effect_time_stun
				.ensure_at_most(-3000, "effect_time_stun")?; //value is set to -3000 on respawn
		} else {
			updated_state.effect_time_stun
				.ensure_not_negative("effect_time_stun")?;
		}
	}
	Ok(())
}

pub(super) fn inspect_effect_time_fear(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.effect_time_fear
		.ensure_not_negative("effect_time_fear")
}

pub(super) fn inspect_effect_time_chill(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.effect_time_chill
		.ensure_not_negative("effect_time_chill")
}

pub(super) fn inspect_effect_time_wind(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.effect_time_wind
		.ensure_within(&(0..=5000), "effect_time_wind")
}

pub(super) fn inspect_show_patch_time(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_occupation(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.occupation
		.ensure_one_of([Warrior, Ranger, Mage, Rogue].as_slice(), "occupation")?;
	inspect_equipment(updated_state, previous_state)
}

pub(super) fn inspect_specialization(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.specialization
		.ensure_one_of([Default, Alternative].as_slice(), "specialization")
}

pub(super) fn inspect_mana_charge(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.mana_charge
		.ensure_at_most(updated_state.mana, "mana_charge")
}

pub(super) fn inspect_unknown24(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_unknown25(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_aim_offset(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	//aim_offset.magnitude().ensure_at_most(60.0, "aim_offset_distance") //todo: account for rounding errors and movement
	Ok(())
}

pub(super) fn inspect_health(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	let maximum = updated_state.maximum_health() + 0.001; //add some tolerance for rounding errors
	updated_state.health
		.ensure_within(&(0.0..=maximum), "health")
}

pub(super) fn inspect_mana(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.mana
		.ensure_within(&(0.0..=1.0), "mana")
	//todo: mana can only increase via:
	//- m1
	//- ninja dodge
	//- blocking
	//- mage passive
	//- camouflage
	//- sniping
	//- stealth (leaving stealth keeps generating mp for a while)
	//- intercept (1 frame to 1.0, then back to 0.0)
}

pub(super) fn inspect_blocking_gauge(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	//it is technically possible to achieve 100% blocking uptime server side + 99% recharge uptime client side legitimately
	//by only blocking for 1ms every time packet construction snapshots the character state
	//while this is inhuman to actually pull off in practice, it does cause false positives every once in a while
	//which is why we unfortunately have to disable this check

	// let blocking_via_shield =//check against former state as the blocking gauge updates with 1 frame delay
	// 	former_state.animation == ShieldM2Charging;
	//
	// let blocking_via_guardians_passive =
	// 	(former_state.combat_class() == GUARDIAN) &&
	// 		former_state.animation
	// 			.present_in(&[
	// 				GreatweaponM2Charging,
	// 				UnarmedM2Charging
	// 			]);
	//
	// let blocking = blocking_via_shield || blocking_via_guardians_passive;
	//
	// let max =
	// 	if blocking {
	// 		former_state.blocking_gauge
	// 	} else { 1.0 };
	//
	// blocking_gauge
	// 	.ensure_within(&(0.0..=max), "blocking_gauge") //todo: negative gauge glitch?

	Ok(())
}

pub(super) fn inspect_multipliers(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.multipliers[Health]     .ensure_exact(&100.0, "multipliers.health")?;
	updated_state.multipliers[AttackSpeed].ensure_exact(&  1.0, "multipliers.attack_speed")?;
	updated_state.multipliers[Damage]     .ensure_exact(&  1.0, "multipliers.damage")?;
	updated_state.multipliers[Resi]       .ensure_exact(&  1.0, "multipliers.resi")?;
	updated_state.multipliers[Armor]      .ensure_exact(&  1.0, "multipliers.armor")
}

pub(super) fn inspect_unknown31(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_unknown32(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_level(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.level
		.ensure_within(&(1..=500), "level")
}

pub(super) fn inspect_experience(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	let maximum = maximum_experience_of(updated_state.level);
	updated_state.experience
		.ensure_within(&(0..maximum), "experience")
}

pub(super) fn inspect_master(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.master
		.ensure_exact(&CreatureId(0), "master")
}

pub(super) fn inspect_unknown36(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_rarity(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.rarity
		.ensure_exact(&0, "rarity")
}

pub(super) fn inspect_unknown38(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_home_zone(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_home(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_zone_to_reveal(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_unknown42(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}

pub(super) fn inspect_consumable(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	if updated_state.consumable.kind == item::Kind::Void {
		return Ok(());
	}
	matches!(updated_state.consumable.kind, item::Kind::Consumable(_))
	 	.ensure("consumable.kind", &updated_state.consumable.kind, "any variant of", "Consumable")?;
	updated_state.consumable.as_formula
		.ensure_exact(&false, "consumable.as_formula")?;
	updated_state.consumable.rarity
		.ensure_exact(&NORMAL, "consumable.rarity")?;
	power_of(updated_state.consumable.level as i32)
		.ensure_within(&(0..=power_of(updated_state.level)), "consumable.power")
}

pub(super) fn inspect_equipment(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	use protocol::packet::common::item::KindDiscriminants::*;
	let allowed_kinds_by_slot = [
		(Slot::Unknown    , [            ].as_slice()),
		(Slot::Neck       , [Amulet      ].as_slice()),
		(Slot::Chest      , [Chest       ].as_slice()),
		(Slot::Feet       , [Boots       ].as_slice()),
		(Slot::Hands      , [Gloves      ].as_slice()),
		(Slot::Shoulder   , [Shoulder    ].as_slice()),
		(Slot::LeftWeapon , [Weapon      ].as_slice()),
		(Slot::RightWeapon, [Weapon      ].as_slice()),
		(Slot::LeftRing   , [Ring        ].as_slice()),
		(Slot::RightRing  , [Ring        ].as_slice()),
		(Slot::Lamp       , [Lamp        ].as_slice()),
		(Slot::Special    , [Special     ].as_slice()),
		(Slot::Pet        , [Pet, PetFood].as_slice()),
	];

	for (slot, allowed) in allowed_kinds_by_slot {
		let item = &updated_state.equipment[slot];
		if item.kind == item::Kind::Void {
			continue; //empty item slots contain uninitialized memory
		}

		let property_name = |literal| { format!("equipment[{slot:?}].{literal}") };

		item.as_formula
			.ensure_exact(&false, &property_name("as_formula"))?;
		item.kind.pipe(KindDiscriminants::from)
			.ensure_one_of(allowed, &property_name("kind"))?;
		item.rarity
			.ensure_at_most(LEGENDARY, &property_name("rarity"))?;
		item.material
			.ensure_one_of(allowed_materials(item.kind, updated_state.occupation), &property_name("material"))?;
		(item.level as i32).pipe(power_of)
			.ensure_within(&(0..=power_of(updated_state.level)), &property_name("power"))?;
		item.spirit_counter
			.ensure_within(&(0..=32), &property_name("spirit_counter"))?;
		//normally only 2h weapons can have more than 16 (up to 32) spirits, but we're tolerating 32 on everyhting due to popularity

		//item.flags
		//item.spirits //tolerating everything due to popularity
		//item.seed
		//	.ensure_not_negative(&format!("equipment[{:?}].seed", slot)) //tolerating negative seeds due to popularity
	}

	updated_state
		.equipment
		.iter()
		.map(|item| {
			let Kind::Weapon(weapon) = item.kind
				else { return 0; };

			if TWO_HANDED_WEAPONS.contains(&weapon) { 2 } else { 1 }
		})
		.sum::<usize>()
		.ensure_at_most(2, "equipment.weapon_hand_count")?;

	Ok(())
}

pub(super) fn inspect_name(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	//character names are serialized as a cstring and thus guaranteed to be comprised of single-byte characters exclusively
	updated_state.name.chars().count().ensure_within(&(1..=15), "name.length")?;
	for (n, character) in updated_state.name.chars().enumerate() {
		character.ensure_within(&('!'..='~'), &format!("name[{n}]"))?;
		//all printable ASCII characters except space (0x20)
		//cubeworld doesn't recognize 0x80+ in character names, only in chat messages
	}

	Ok(())
}

pub(super) fn inspect_skill_tree(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	for skill in Skill::iter() {
		updated_state.skill_tree[skill]
			.ensure_not_negative(&format!("skill_tree.{skill:?}"))?;
	}
	updated_state.skill_tree.iter().sum::<i32>()
		.ensure_at_most((updated_state.level - 1) * 2, "skill_tree.total")
}

pub(super) fn inspect_mana_cubes(previous_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	updated_state.mana_cubes.ensure_not_negative("mana_cubes")
}