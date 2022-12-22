use boolinator::Boolinator;

use protocol::nalgebra::{Point3, Vector3};
use protocol::packet::common::{CreatureId, EulerAngles, Hitbox, Item, Race};
use protocol::packet::common::Race::*;
use protocol::packet::creature_update::{Affiliation, Animation, Appearance, CombatClassMajor, CombatClassMinor, CreatureFlag, Equipment, Multipliers, PhysicsFlag, SkillTree};
use protocol::packet::creature_update::Animation::*;
use protocol::packet::creature_update::CombatClassMajor::*;
use protocol::packet::creature_update::CombatClassMinor::*;
use protocol::utils::constants::{animations, PLAYABLE_RACES};
use protocol::utils::constants::animations::{abilities, m1, m2};
use protocol::utils::constants::combat_classes::*;
use protocol::utils::constants::item_types::*;
use protocol::utils::flagset::{FlagSet16, FlagSet32};

use crate::anti_cheat;
use crate::anti_cheat::*;
use crate::creature::Creature;

pub(super) fn inspect_position(position: &Point3<i64>, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_rotation(rotation: &EulerAngles, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	//usually 0, except
	//- rounding errors
	//- 60f..=0 when swimming (or shortly afterwards)
	//- 20f when teleporting
	rotation.pitch
		.is_finite()
		.ok_or("rotation.yaw wasn't finite")?;
	rotation.roll
		.ensure_within(&(-90f32..=90f32), "rotation.roll")?;
	rotation.yaw//normally -180..=180, but over-/underflows while attacking
		.is_finite()
		.ok_or("rotation.yaw wasn't finite".to_string())
}
pub(super) fn inspect_velocity(velocity: &Vector3<f32>, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_acceleration(acceleration: &Vector3<f32>, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
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
pub(super) fn inspect_velocity_extra(velocity_extra: &Vector3<f32>, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	let (max_xy, max_z): (f32, f32) =
		match updated_state.combat_class_major {
			Ranger => (35.0, 17.0),
			_      => ( 0.1,  0.0)//0.1 because the game doesnt reset all the way to 0
		};

	velocity_extra.xy()
		.magnitude()
		.ensure_at_most(max_xy, "retreat_horizontal_speed")?;
	velocity_extra.z
		.ensure_within(&(0.0..=max_z), "")
}
pub(super) fn inspect_head_tilt(head_tilt: &f32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	head_tilt
		.ensure_within(&(-32.5..=45.0), "head_tilt")//negative when attacking downwards
}
pub(super) fn inspect_flags_physics(flags_physics: &FlagSet32<PhysicsFlag>, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_affiliation(affiliation: &Affiliation, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	affiliation
		.ensure_exact(&Affiliation::Player, "affiliation")
}
pub(super) fn inspect_race(race: &Race, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	race.ensure_one_of(PLAYABLE_RACES.as_slice(), "")
}
pub(super) fn inspect_animation(animation: &Animation, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	let abilities =
		match updated_state.combat_class_major {
			Warrior => &abilities::WARRIOR[..],
			Ranger  => &abilities::RANGER[..],
			Mage    => match updated_state.combat_class_minor {
				Alternative => &abilities::WATER_MAGE[..],
				Default | _ => &abilities::FIRE_MAGE[..],
			}
			Rogue   => match updated_state.combat_class_minor {
				Default         => &abilities::ASSASSIN[..],
				Alternative | _ => &abilities::NINJA[..],//no, this is not a bug. the game is actually that inconsistent
			}
			_ => &[][..]
		};

	let right = updated_state.equipment.right_weapon.item_type();
	let left  = updated_state.equipment.left_weapon.item_type();

	let left_handed = left.present_in(&[BOW, CROSSBOW]);

	let (mainhand, offhand) =
		if left_handed { (left, right) }
		else           { (right, left) };

	let (m1, m2) = match mainhand {
		GREATSWORD |
		GREATAXE   |
		GREATMACE  |
		PITCHFORK => (&m1::GREATWEAPON[..], &m2::GREATWEAPON[..]),
		DAGGER    => (&m1::DAGGER[..]     , &m2::DAGGER[..]),
		FIST      => (&m1::UNARMED[..]    , &m2::UNARMED[..]),//use redirecting constants?
		LONGSWORD => (&m1::LONGSWORD[..]  , &m2::LONGSWORD[..]),
		BOW       => (&m1::BOW[..]        , &m2::BOW[..]),
		CROSSBOW  => (&m1::CROSSBOW[..]   , &m2::CROSSBOW[..]),
		BOOMERANG => (&m1::BOOMERANG[..]  , &m2::BOOMERANG[..]),
		STAFF     => match updated_state.combat_class_minor {
			Alternative => (&m1::STAFF_WATER[..]   , &m2::STAFF_WATER[..]),
			_           => (&m1::STAFF_FIRE[..]    , &m2::STAFF_FIRE[..])
		},
		WAND      => match updated_state.combat_class_minor {
			Alternative => (&m1::WAND_WATER[..]    , &m2::WAND_WATER[..]),
			_           => (&m1::WAND_FIRE[..]     , &m2::WAND_FIRE[..])
		},
		BRACELET  => match updated_state.combat_class_minor {
			Alternative => (&m1::BRACELET_WATER[..], &m2::BRACELET_WATER[..]),
			_           => (&m1::BRACELET_FIRE[..] , &m2::BRACELET_FIRE[..])
		},
		NONE      => {
			let (mainhand_m1, mainhand_m2) = match updated_state.combat_class() {
				FIRE_MAGE  => (&m1::BRACELET_FIRE[..] , &m2::BRACELET_FIRE[..]),
				WATER_MAGE => (&m1::BRACELET_WATER[..], &m2::BRACELET_WATER[..]),
				_          => (&m1::UNARMED[..]       , &m2::UNARMED[..])
			};
			let m2 =
				match offhand {
					SHIELD => &m2::SHIELD[..],
					_      => mainhand_m2
				};

			(mainhand_m1, m2)
		},
//		SWORD | AXE | MACE |
//		SHIELD|
//		ARROW | QUIVER | PICKAXE | TORCH
		_ => match offhand {
			SHIELD => (&m1::SHIELD[..]   , &m2::SHIELD[..]),
			_      => (&m1::DUALWIELD[..], &m2::UNARMED[..])//use redirecting constant?
		}
	};

	let allowed_animations = [&animations::GENERAL[..], abilities, m1, m2].concat();

	animation
		.ensure_one_of(&allowed_animations, "animation")?;

	Ok(())
}
pub(super) fn inspect_animation_time(animation_time: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	const TIMELESS_ANIMATIONS: [Animation; 6] = [
		Idle,
		Stealth,
		Boat,
		Sitting,
		PetFoodPresent,
		Sleeping
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
pub(super) fn inspect_combo(combo: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	combo.ensure_not_negative("combo")
}
pub(super) fn inspect_hit_time_out(hit_time_out: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	hit_time_out.ensure_not_negative("hit_time_out")
	//todo
//	if (this <= previous.hitTimeOut) {
//		lastHitTime[id] = System.currentTimeMillis() - this
//	} else {
//		val n = System.currentTimeMillis() - this - lastHitTime[id]!!
//		if (id.value == 1L) println(n)
//		abs(n).expectMaximum(2000, "hitTimeOut.clockdesync")
//	}
//
//	if (this == previous.hitTimeOut) {
//		//join packet, ignore because lag
//	} else if (this < previous.hitTimeOut) {
//		lastHitTime[id] = System.currentTimeMillis() - this
//	} else if (lastHitTime[id] == null) {
//		//no reference point generated yet
//	} else {
//		val n = System.currentTimeMillis() - this - lastHitTime[id]!!
//		if (id.value == 1L) println(n)
//		abs(n).expectMaximum(2000, "hitTimeOut.clockdesync")
//	}
}
pub(super) fn inspect_appearance(appearance: &Appearance, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	appearance.flags.ensure_exact(&core::default::Default::default(), "appearance.flags")?;

	appearance.tail_model.ensure_exact(&-1, "asdf")?;
	appearance.shoulder2model.ensure_exact(&-1, "asdf")?;
	appearance.wing_model.ensure_exact(&-1, "asdf")?;

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
pub(super) fn inspect_flags(flags: &FlagSet16<CreatureFlag>, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_effect_time_dodge(effect_time_dodge: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	effect_time_dodge.ensure_within(&(0..=600), "effect_time_dodge")
}
pub(super) fn inspect_effect_time_stun(effect_time_stun: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	//todo: ensure positive when increased
	Ok(())
}
pub(super) fn inspect_effect_time_fear(effect_time_fear: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	effect_time_fear.ensure_not_negative("effect_time_fear")
}
pub(super) fn inspect_effect_time_chill(effect_time_chill: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	effect_time_chill.ensure_not_negative("effect_time_chill")
}
pub(super) fn inspect_effect_time_wind(effect_time_wind: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	effect_time_wind.ensure_within(&(0..=5000), "effect_time_wind")
}
pub(super) fn inspect_show_patch_time(show_patch_time: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_combat_class_major(combat_class_major: &CombatClassMajor, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	combat_class_major.ensure_one_of([Warrior, Ranger, Mage, Rogue].as_slice(), "combat_class_major")
	//todo: recheck gear
}
pub(super) fn inspect_combat_class_minor(combat_class_minor: &CombatClassMinor, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	combat_class_minor.ensure_one_of([Default, Alternative].as_slice(), "combat_class_minor")
}
pub(super) fn inspect_mana_charge(mana_charge: &f32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	mana_charge.ensure_at_most(updated_state.mana, "mana_charge")
}
pub(super) fn inspect_unknown24(unknown24: &[f32; 3], former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_unknown25(unknown25: &[f32; 3], former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_aim_offset(aim_offset: &Point3<f32>, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	//aim_offset.magnitude().ensure_at_most(60.0, "aim_offset_distance") //todo: account for rounding errors and movement
	Ok(())
}
pub(super) fn inspect_health(health: &f32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	//todo: calculate max hp
	Ok(())
}
pub(super) fn inspect_mana(mana: &f32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
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
pub(super) fn inspect_blocking_gauge(blocking_gauge: &f32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	let blocking_via_shield =//check against former state as the blocking gauge updates with 1 frame delay
		former_state.animation == ShieldM2Charging;

	let blocking_via_guardians_passive =
		(former_state.combat_class() == GUARDIAN) &&
			former_state.animation
				.present_in(&[
					DualWieldM2Charging,
					GreatweaponM2Charging,
					UnarmedM2Charging
				]);

	let blocking = blocking_via_shield || blocking_via_guardians_passive;

	let max =
		if blocking {
			former_state.blocking_gauge
		} else {
			1.0
		};

	blocking_gauge
		.ensure_within(&(0.0..=max), "blocking_gauge") //todo: negative gauge glitch?
}
pub(super) fn inspect_multipliers(multipliers: &Multipliers, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	multipliers.health      .ensure_exact(&100.0, "multipliers.health")?;
	multipliers.attack_speed.ensure_exact(&  1.0, "multipliers.attack_speed")?;
	multipliers.damage      .ensure_exact(&  1.0, "multipliers.damage")?;
	multipliers.resi        .ensure_exact(&  1.0, "multipliers.resi")?;
	multipliers.armor       .ensure_exact(&  1.0, "multipliers.armor")
}
pub(super) fn inspect_unknown31(unknown31: &i8, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_unknown32(unknown32: &i8, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_level(level: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	level.ensure_within(&(1..=500), "level")
}
pub(super) fn inspect_experience(experience: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	let max = 9999;//todo: calc max xp based on lvl
	experience.ensure_within(&(0..=max), "experience")
}
pub(super) fn inspect_master(master: &CreatureId, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	master
		.ensure_exact(&CreatureId(0), "master")
}
pub(super) fn inspect_unknown36(unknown36: &i64, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_power_base(power_base: &i8, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	power_base
		.ensure_exact(&0, "power_base")
}
pub(super) fn inspect_unknown38(unknown38: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_home_zone(home_zone: &Point3<i32>, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_home(home: &Point3<i64>, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_zone_to_reveal(zone_to_reveal: &Point3<i32>, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_unknown42(unknown42: &i8, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	Ok(())
}
pub(super) fn inspect_consumable(consumable: &Item, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	//todo
//	if (it.typeMajor == Item.Type.Major.None) return@let
//
//		it.typeMajor.expect(Item.Type.Major.Food, "consumable.typeMajor")
//	it.rarity.expect(Item.Rarity.Normal, "consumable.rarity")
//
//	val powerAllowed = Utils.computePower(current.level)
//	val powerActual = Utils.computePower(it.level.toInt())
//
//	powerActual.expectIn(1..=powerAllowed, "consumable.level")
//	it.spiritCounter.expect(0, "consumable.spiritCounter")
	Ok(())
}
pub(super) fn inspect_equipment(equipment: &Equipment, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
//	mapOf(
//		it::unknown     to setOf(Item.Type.Major.None),
//		it::neck        to setOf(Item.Type.Major.Amulet),
//		it::chest       to setOf(Item.Type.Major.Chest),
//		it::feet        to setOf(Item.Type.Major.Boots),
//		it::hands       to setOf(Item.Type.Major.Gloves),
//		it::shoulder    to setOf(Item.Type.Major.Shoulder),
//		it::leftWeapon  to setOf(Item.Type.Major.Weapon),
//		it::rightWeapon to setOf(Item.Type.Major.Weapon),
//		it::leftRing    to setOf(Item.Type.Major.Ring),
//		it::rightRing   to setOf(Item.Type.Major.Ring),
//		it::lamp        to setOf(Item.Type.Major.Lamp),
//		it::special     to setOf(Item.Type.Major.Special),
//		it::pet         to setOf(Item.Type.Major.Pet, Item.Type.Major.PetFood)
//	).filterNot { it.key.get().typeMajor == Item.Type.Major.None }
//		.forEach {
//		val item = it.key.get()
//		val prefix = "equipment." + it.key.name
//
//		item.typeMajor.expectIn(it.value, "$prefix.typeMajor")
//
//		val classMajor = current.combatClassMajor
//		val allowedItemMaterials = when (item.typeMajor) {
//			Item.Type.Major.Weapon -> {
//				item.typeMinor.expectIn(getAllowedWeaponTypes(classMajor), "$prefix.typeMinor")
//				allowedWeaponMaterials[item.typeMinor]!!
//			}
//			Item.Type.Major.Chest,
//			Item.Type.Major.Boots,
//			Item.Type.Major.Gloves,
//			Item.Type.Major.Shoulder -> getAllowedMaterialsArmor(classMajor)
//
//			Item.Type.Major.Amulet,
//			Item.Type.Major.Ring -> allowedMaterialsAccessories
//
//			Item.Type.Major.Special -> {
//				item.typeMinor.expectIn(subTypesSpecial, "$prefix.typeMinor")
//				setOf(Item.Material.Wood)
//			}
//			Item.Type.Major.Lamp -> setOf(Item.Material.Iron)
//			else -> setOf(Item.Material.None)
//		}
//		item.material.expectIn(allowedItemMaterials, "$prefix.material")
//		//item.randomSeed.expectMinimum(0, "$prefix.randomSeed")
//		item.recipe.expect(Item.Type.Major.None, "$prefix.recipe")
//		item.rarity.expectIn(allowedRarities, "$prefix.rarity")
//
//		val powerAllowed = Utils.computePower(current.level)
//		val powerActual = Utils.computePower(item.level.toInt())
//		powerActual.expectIn(1..=powerAllowed, "$prefix.level")
//
//		val spiritLimit = 32//if (item.typeMajor == Item.Type.Major.Weapon) getWeaponHandCount(item.typeMinor) * 16 else 0
//		item.spiritCounter.expectIn(0..=spiritLimit, "$prefix.spiritCounter")
//
//		val allowedSpiritMaterials = setOf(
//			Item.Material.Fire,
//				Item.Material.Unholy,
//			Item.Material.IceSpirit,
//			Item.Material.Wind,
//			item.material
//			)
//		item.spirits.take(item.spiritCounter).forEachIndexed { index, spirit ->
//			spirit.material.expectIn(allowedSpiritMaterials, "$prefix.spirit#$index.material")
//			spirit.level.toInt().expectIn(1..=item.level, "$prefix.spirit#$index.level")
//		}
//	}
//	val r = if (it.rightWeapon == Item.void) 0 else getWeaponHandCount(it.rightWeapon.typeMinor)
//	val l = if (it.leftWeapon == Item.void) 0 else getWeaponHandCount(it.leftWeapon.typeMinor)
//		(r + l).expectMaximum(2, "equipment.dualwield")
//	//ranger can hold 2h weapon in either hand
//
//	inspect(CreatureUpdate(id = id, animation = current.animation), current)?.let {
//		throw CheaterException(it)
//	}
	Ok(())
}
pub(super) fn inspect_name(name: &String, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	name.as_bytes().len().ensure_within(&(1..=15), "name.length")
	//todo: limit characters to what the default font can display
}
pub(super) fn inspect_skill_tree(skill_tree: &SkillTree, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	let skills = [//todo: implement .iter() for SkillTree directly?
		skill_tree.pet_master,
		skill_tree.pet_riding,
		skill_tree.sailing,
		skill_tree.climbing,
		skill_tree.hang_gliding,
		skill_tree.swimming,
		skill_tree.ability1,
		skill_tree.ability2,
		skill_tree.ability3,
		skill_tree.ability4,
		skill_tree.ability5,
	];
	for skill in skills {
		skill.ensure_not_negative("skill")?;//todo: individual names
	}
	skills.iter().sum::<i32>().ensure_at_most((updated_state.level - 1) * 2, "skillPoints.total")
}
pub(super) fn inspect_mana_cubes(mana_cubes: &i32, former_state: &Creature, updated_state: &Creature) -> anti_cheat::Result {
	mana_cubes.ensure_not_negative("mana_cubes")
}