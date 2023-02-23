#![allow(unused_variables)]

use std::time::Duration;

use boolinator::Boolinator;
use strum::IntoEnumIterator;

use protocol::nalgebra::{Point3, Vector3};
use protocol::packet::common::{CreatureId, EulerAngles, Hitbox, Item, item, Race};
use protocol::packet::common::item::Kind::*;
use protocol::packet::common::item::Material;
use protocol::packet::common::Race::*;
use protocol::packet::creature_update::{Affiliation, Animation, Appearance, CreatureFlag, Equipment, Multipliers, Occupation, PhysicsFlag, SkillTree, Specialization};
use protocol::packet::creature_update::Animation::*;
use protocol::packet::creature_update::equipment::Slot;
use protocol::packet::creature_update::multipliers::Multiplier::*;
use protocol::packet::creature_update::Occupation::*;
use protocol::packet::creature_update::skill_tree::Skill;
use protocol::packet::creature_update::Specialization::*;
use protocol::utils::{maximum_experience_of, power_of};
use protocol::utils::constants::combat_classes::*;
use protocol::utils::constants::PLAYABLE_RACES;
use protocol::utils::constants::rarity::*;
use protocol::utils::flagset::{FlagSet16, FlagSet32};

use crate::addons::anti_cheat;
use crate::addons::anti_cheat::*;
use crate::addons::anti_cheat::creature_update::animation::animations_avilable_with;
use crate::addons::anti_cheat::creature_update::equipment::allowed_materials;
use crate::server::creature::Creature;

mod animation;
mod equipment;

pub(super) fn inspect_position(position: &Point3<i64>, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_rotation(rotation: &EulerAngles, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	//usually 0, except
	//- rounding errors
	//- 60f..=0 when swimming (or shortly afterwards)
	//- 20f when teleporting
	rotation.pitch
		.is_finite()
		.ok_or("rotation.yaw wasn't finite")?;
	rotation.roll
		.ensure_within(&(-90.0..=90.0), "rotation.roll")?;
	rotation.yaw//normally -180..=180, but over-/underflows while attacking
		.is_finite()
		.ok_or("rotation.yaw wasn't finite".to_string())
}
pub(super) fn inspect_velocity(velocity: &Vector3<f32>, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_acceleration(acceleration: &Vector3<f32>, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	let limit_xy = Vector3::<f32>::new(80.0, 80.0, 0.0).magnitude() + 0.00001; //113,1370849898476; //todo: would epsilon suffice?
	let actual_xy = acceleration.xy().magnitude();
	if !updated_state.flags.get(CreatureFlag::Gliding) {
		actual_xy.ensure_within(&(0.0..=limit_xy), "acceleration.horizontal")?;
	}
	if updated_state.flags_physics.get(PhysicsFlag::Swimming) {
		acceleration.z.ensure_within(&(-80.0..=80.0), "acceleration.vertical")
	} else if updated_state.flags.get(CreatureFlag::Climbing) {
		acceleration.z.ensure_one_of(&[-16.0, 0.0, 16.0], "acceleration.vertical")
	} else {
		acceleration.z.ensure_exact(&0.0, "acceleration.vertical")
	}
}
pub(super) fn inspect_velocity_extra(velocity_extra: &Vector3<f32>, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	let (max_xy, max_z): (f32, f32) =
		match updated_state.occupation {
			Ranger => (35.0, 17.0),
			_      => ( 0.1,  0.0)//0.1 because the game doesnt reset all the way to 0
		};

	velocity_extra.xy()
		.magnitude()
		.ensure_at_most(max_xy, "retreat_horizontal_speed")?;
	velocity_extra.z
		.ensure_within(&(0.0..=max_z), "")
}
pub(super) fn inspect_head_tilt(head_tilt: &f32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	head_tilt
		.ensure_within(&(-32.5..=45.0), "head_tilt")//negative when attacking downwards
}
pub(super) fn inspect_flags_physics(flags_physics: &FlagSet32<PhysicsFlag>, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_affiliation(affiliation: &Affiliation, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Affiliation::iter().any(|it| *affiliation == it).ok_or("invalid affiliation".to_string())?;//todo: safety measure until data validation is implemented
	affiliation
		.ensure_exact(&Affiliation::Player, "affiliation")
}
pub(super) fn inspect_race(race: &Race, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Race::iter().any(|it| *race == it).ok_or("invalid race".to_string())?;//todo: safety measure until data validation is implemented
	race.ensure_one_of(PLAYABLE_RACES.as_slice(), "")
}
pub(super) fn inspect_animation(animation: &Animation, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Animation::iter().any(|it| *animation == it).ok_or("invalid animation".to_string())?;//todo: safety measure until data validation is implemented
	let allowed_animations = animations_avilable_with(updated_state.combat_class(), &updated_state.equipment);

	animation
		.ensure_one_of(&allowed_animations, "animation")
}
pub(super) fn inspect_animation_time(animation_time: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	const TIMELESS_ANIMATIONS: [Animation; 12] = [
		Idle,
		Stealth,
		Sailing,
		Sitting,
		PetFoodPresent,
		Sleeping,
		//todo: separate?
		ShieldM2Charging,
		GreatweaponM2Charging,
		BoomerangM2Charging,
		CrossbowM2Charging,
		UnarmedM2Charging,
		BowM2Charging
		//todo: some unused animations are timeless as well
	];

	animation_time.ensure_not_negative("animation time")?;

	if !updated_state.animation.present_in(&TIMELESS_ANIMATIONS) {
		animation_time.ensure_at_most(10_000, "animation time")?;
	};

	if *animation_time < former_state.animation_time && updated_state.animation == FireExplosionShort {
		//todo: detect fire spam
	}

	Ok(())
}
pub(super) fn inspect_combo(combo: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	combo.ensure_not_negative("combo")
}
pub(super) fn inspect_combo_timeout(combo_timeout: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	combo_timeout.ensure_not_negative("combo_timeout")?;

	if *combo_timeout <= former_state.hit_time_out { //equal incase of seed change lag
		ac_data.last_combo_update = Some(Instant::now() - Duration::from_millis(*combo_timeout as u64));
		return Ok(());
	}


	let expected = ac_data.last_combo_update
		.expect("unreachable: guaranteed to be initialized on join")
		.elapsed()
		.as_millis() as i32;

	combo_timeout.abs_diff(expected)
		.ensure_at_most(2000, "combo_timeout.clockdesync")
}
pub(super) fn inspect_appearance(appearance: &Appearance, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	appearance.flags.ensure_exact(&core::default::Default::default(), "appearance.flags")?;

	appearance.tail_model.ensure_exact(&-1, "appearance.tail_model")?;
	appearance.shoulder2model.ensure_exact(&-1, "appearance.shoulder2model")?;
	appearance.wing_model.ensure_exact(&-1, "appearance.wing_model")?;

	appearance.hand_size.ensure_exact(&1.0, "appearance.hand_size")?;
	appearance.foot_size.ensure_exact(&0.98, "appearance.footSize")?;
	appearance.tail_size.ensure_exact(&0.8, "appearance.tailSize")?;
	appearance.shoulder2size.ensure_exact(&1.0, "appearance.shoulder1Size")?;
	appearance.wing_size.ensure_exact(&1.0, "appearance.wingSize")?;

	appearance.body_offset.ensure_exact(&Point3::new(0.0, 0.0, -5.0), "appearance.bodyOffset")?;
	appearance.head_offset.ensure_exact(
		&if updated_state.race == OrcFemale {
			Point3::new(0.0, 1.5, 4.0)
		} else {
			Point3::new(0.0, 0.5, 5.0)
		},
		"appearance.headOffset"
	)?;
	appearance.hand_offset.ensure_exact(&Point3::new(6.0, 0.0,  0.0), "appearance.handOffset")?;
	appearance.foot_offset.ensure_exact(&Point3::new(3.0, 1.0,-10.5), "appearance.footOffset")?;
	appearance.tail_offset.ensure_exact(&Point3::new(0.0,-8.0,  2.0), "appearance.tailOffset")?;
	appearance.wing_offset.ensure_exact(&Point3::new(0.0, 0.0,  0.0), "appearance.wingOffset")?;

	appearance.body_rotation.ensure_exact(&0.0, "appearance.bodyRotation")?;
	appearance.hand_rotation.ensure_exact(&core::default::Default::default(), "appearance.handRotation")?;
	appearance.feet_rotation.ensure_exact(&0.0, "appearance.feetRotation")?;
	appearance.wing_rotation.ensure_exact(&0.0, "appearance.wingRotation")?;
	appearance.tail_rotation.ensure_exact(&0.0, "appearance.tail_rotation")?;

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

	appearance.creature_size.ensure_exact (&allowed_creature_size, "appearance.creature.Size")?;
	appearance.head_model   .ensure_within(&allowed_head_model   , "appearance.headModel")?;
	appearance.hair_model   .ensure_within(&allowed_hair_model   , "appearance.hairModel")?;
	appearance.hand_model   .ensure_within(&allowed_hand_model   , "appearance.handModel")?;
	appearance.foot_model   .ensure_exact (&allowed_foot_model   , "appearance.footModel")?;
	appearance.body_model   .ensure_exact (&allowed_body_model   , "appearance.bodyModel")?;
	appearance.head_size    .ensure_exact (&allowed_head_size    , "appearance.headSize")?;
	appearance.body_size    .ensure_exact (&allowed_body_size    , "appearance.bodySize")?;
	appearance.shoulder1size.ensure_exact (&allowed_shoulder1size, "appearance.shoulder2Size")?;
	appearance.weapon_size  .ensure_exact (&allowed_weapon_size  , "appearance.weaponSize")

}
pub(super) fn inspect_flags(flags: &FlagSet16<CreatureFlag>, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	flags.get(CreatureFlag::FriendlyFire)
		.ensure_exact(&false, "flags[FriendlyFire]")?;

	if updated_state.combat_class() != SNIPER {
		flags.get(CreatureFlag::Sniping)
			.ensure_exact(&false, "flags[Sniping]")?;
	}

	if updated_state.equipment[Slot::Lamp].kind == item::Kind::Void {
		flags.get(CreatureFlag::Lamp)
			.ensure_exact(&false, "flags[Lamp]")?;
	}
	Ok(())
}
pub(super) fn inspect_effect_time_dodge(effect_time_dodge: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	effect_time_dodge.ensure_within(&(0..=600), "effect_time_dodge")
}
pub(super) fn inspect_effect_time_stun(effect_time_stun: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	if *effect_time_stun > former_state.effect_time_stun {
		effect_time_stun.ensure_not_negative("effect_time_stun")?;
	}
	Ok(())
}
pub(super) fn inspect_effect_time_fear(effect_time_fear: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	effect_time_fear.ensure_not_negative("effect_time_fear")
}
pub(super) fn inspect_effect_time_chill(effect_time_chill: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	effect_time_chill.ensure_not_negative("effect_time_chill")
}
pub(super) fn inspect_effect_time_wind(effect_time_wind: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	effect_time_wind.ensure_within(&(0..=5000), "effect_time_wind")
}
pub(super) fn inspect_show_patch_time(show_patch_time: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_occupation(occupation: &Occupation, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	occupation.present_in(&[Warrior, Ranger, Mage, Rogue]).ok_or("invalid combat_class_major".to_string())?;//todo: safety measure until data validation is implemented
	occupation.ensure_one_of([Warrior, Ranger, Mage, Rogue].as_slice(), "combat_class_major")?;
	inspect_equipment(&updated_state.equipment, former_state, updated_state, ac_data)
}
pub(super) fn inspect_specialization(specialization: &Specialization, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	specialization.present_in(&[Default, Alternative]).ok_or("invalid combat_class_minor".to_string())?;//todo: safety measure until data validation is implemented
	specialization.ensure_one_of([Default, Alternative].as_slice(), "combat_class_minor")
}
pub(super) fn inspect_mana_charge(mana_charge: &f32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	mana_charge.ensure_at_most(updated_state.mana, "mana_charge")
}
pub(super) fn inspect_unknown24(unknown24: &[f32; 3], former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_unknown25(unknown25: &[f32; 3], former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_aim_offset(aim_offset: &Point3<f32>, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	//aim_offset.magnitude().ensure_at_most(60.0, "aim_offset_distance") //todo: account for rounding errors and movement
	Ok(())
}
pub(super) fn inspect_health(health: &f32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	let maximum = updated_state.maximum_health() + 0.001; //add some tolerance for rounding errors
	health.ensure_within(&(0.0..=maximum), "health")
}
pub(super) fn inspect_mana(mana: &f32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	mana.ensure_within(&(0.0..=1.0), "mana")
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
pub(super) fn inspect_blocking_gauge(blocking_gauge: &f32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	let blocking_via_shield =//check against former state as the blocking gauge updates with 1 frame delay
		former_state.animation == ShieldM2Charging;

	let blocking_via_guardians_passive =
		(former_state.combat_class() == GUARDIAN) &&
			former_state.animation
				.present_in(&[
					GreatweaponM2Charging,
					UnarmedM2Charging
				]);

	let blocking = blocking_via_shield || blocking_via_guardians_passive;

	let max =
		if blocking { former_state.blocking_gauge }
		else        { 1.0 };

	blocking_gauge
		.ensure_within(&(0.0..=max), "blocking_gauge") //todo: negative gauge glitch?
}
pub(super) fn inspect_multipliers(multipliers: &Multipliers, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	multipliers[Health]     .ensure_exact(&100.0, "multipliers.health")?;
	multipliers[AttackSpeed].ensure_exact(&  1.0, "multipliers.attack_speed")?;
	multipliers[Damage]     .ensure_exact(&  1.0, "multipliers.damage")?;
	multipliers[Resi]       .ensure_exact(&  1.0, "multipliers.resi")?;
	multipliers[Armor]      .ensure_exact(&  1.0, "multipliers.armor")
}
pub(super) fn inspect_unknown31(unknown31: &i8, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_unknown32(unknown32: &i8, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_level(level: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	level.ensure_within(&(1..=500), "level")
}
pub(super) fn inspect_experience(experience: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	let maximum = maximum_experience_of(updated_state.level);
	experience.ensure_within(&(0..maximum), "experience")
}
pub(super) fn inspect_master(master: &CreatureId, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	master
		.ensure_exact(&CreatureId(0), "master")
}
pub(super) fn inspect_unknown36(unknown36: &i64, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_rarity(rarity: &u8, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	rarity
		.ensure_exact(&0, "rarity")
}
pub(super) fn inspect_unknown38(unknown38: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_home_zone(home_zone: &Point3<i32>, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_home(home: &Point3<i64>, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_zone_to_reveal(zone_to_reveal: &Point3<i32>, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_unknown42(unknown42: &i8, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_consumable(consumable: &Item, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	if consumable.kind == Void {
		return Ok(());
	}
	matches!(consumable.kind, Consumable(_))
		.ok_or("illegal consumable.kind")?;//todo: safety measure until data validation is implemented
	 matches!(consumable.kind, Consumable(_))
	 	.ensure("consumable.kind", &consumable.kind, "any variant of", "Consumable")?;
	consumable.rarity
		.ensure_exact(&NORMAL, "consumable.rarity")?;
	power_of(consumable.level as i32)
		.ensure_within(&(0..=power_of(updated_state.level)), "consumable.power")
}
pub(super) fn inspect_equipment(equipment: &Equipment, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	//todo: copypasta
	let invariant_slots = [
		(Slot::Unknown  , item::Kind::Void),
		(Slot::Neck     , item::Kind::Amulet),
		(Slot::Chest    , item::Kind::Chest),
		(Slot::Feet     , item::Kind::Boots),
		(Slot::Hands    , item::Kind::Gloves),
		(Slot::Shoulder , item::Kind::Shoulder),
		(Slot::LeftRing , item::Kind::Ring),
		(Slot::RightRing, item::Kind::Ring),
		(Slot::Lamp     , item::Kind::Lamp)
	];

	for (slot, allowed_kind) in invariant_slots {
		equipment[slot].kind.present_in(&[Void, allowed_kind]).ok_or(format!("illegal equipment[{:?}].kind", slot))?;//todo: safety measure until data validation is implemented
		equipment[slot].kind.ensure_one_of(&[Void, allowed_kind], &format!("equipment[{:?}].kind", slot))?;
	}

	//todo: safety measure until data validation is implemented
	matches!(equipment[Slot::LeftWeapon].kind, Void | Weapon(_)).ok_or(format!("illegal equipment[{:?}].kind", Slot::LeftWeapon))?;
	matches!(equipment[Slot::RightWeapon].kind, Void | Weapon(_)).ok_or(format!("illegal equipment[{:?}].kind", Slot::RightWeapon))?;
	matches!(equipment[Slot::Special].kind, Void | Special(_)).ok_or(format!("illegal equipment[{:?}].kind", Slot::Special))?;
	matches!(equipment[Slot::Pet].kind, Void | Pet(_) | PetFood(_)).ok_or(format!("illegal equipment[{:?}].kind", Slot::Pet))?;

	matches!(equipment[Slot::LeftWeapon].kind, Void | Weapon(_))
		.ensure(
			"equipment[LeftWeapon].kind",
			&equipment[Slot::LeftWeapon].kind,
			"any variant of",
			"Weapon"
		)?;
	matches!(equipment[Slot::RightWeapon].kind, Void | Weapon(_))
		.ensure(
			"equipment[RightWeapon].kind",
			&equipment[Slot::RightWeapon].kind,
			"any variant of",
			"Weapon"
		)?;
	matches!(equipment[Slot::Special].kind, Void | Special(_))
		.ensure(
			"equipment[Special].kind",
			&equipment[Slot::Special].kind,
			"any variant of",
			"Special"
		)?;
	matches!(equipment[Slot::Pet].kind, Void | Pet(_) | PetFood(_))
		.ensure(
			"equipment[Pet].kind",
			&equipment[Slot::Pet].kind,
			"any variant of",
			"Pet or PetFood"
		)?;

	for slot in Slot::iter() {
		let item = &equipment[slot];
		if item.kind == Void {
			continue; //empty item slots contain uninitialized memory
		}

		//todo: safety measure until data validation is implemented
		Material::iter().any(|material| item.material == material).ok_or(format!("invalid equipment[{:?}].kind", slot))?;

		//item.seed.ensure_not_negative(&format!("equipment[{:?}].seed", slot)) //tolerating negative seeds due to popularity
		//item._recipe.ensure_exact(&Void, &format!("equipment[{:?}].recipe", slot))?;
		//item.minus_modifier
		item.rarity.ensure_at_most(LEGENDARY, &format!("equipment[{:?}].rarity", slot))?; //todo: crashes for rarity 6+
		let allowed_materials = allowed_materials(item.kind, updated_state.occupation);
		item.material.ensure_one_of(allowed_materials, &format!("equipment[{:?}].material", slot))?;
		//item.flags
		power_of(item.level as i32)
			.ensure_within(&(0..=power_of(updated_state.level)), &format!("equipment[{:?}].power", slot))?;
		//item.spirits //tolerating everything due to popularity
		item.spirit_counter.ensure_within(&(0..=32), &format!("equipment[{:?}].spirit_counter", slot))?;//normally only 2h weapons can have more than 16 (up to 32) spirits, but we're tolerating 32 on everyhting due to popularity
	}

	Ok(())
}
pub(super) fn inspect_name(name: &String, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	name.as_bytes().len().ensure_within(&(1..=15), "name.length")
	//todo: limit characters to what the default font can display
}
pub(super) fn inspect_skill_tree(skill_tree: &SkillTree, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	for skill in Skill::iter() {
		skill_tree[skill].ensure_not_negative(&format!("skill_tree.{:?}", skill))?;
	}
	skill_tree.iter().sum::<i32>().ensure_at_most((updated_state.level - 1) * 2, "skill_tree.total")
}
pub(super) fn inspect_mana_cubes(mana_cubes: &i32, former_state: &Creature, updated_state: &Creature, ac_data: &mut PlayerACData) -> anti_cheat::Result {
	mana_cubes.ensure_not_negative("mana_cubes")
}