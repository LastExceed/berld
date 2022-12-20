use std::ops::RangeBounds;

use boolinator::Boolinator;

use protocol::nalgebra::{Point3, Vector3};
use protocol::packet::common::{CreatureId, EulerAngles, Item, Race};
use protocol::packet::common::Race::*;
use protocol::packet::creature_update::{Affiliation, Animation, Appearance, CombatClassMajor, CombatClassMinor, CreatureFlag, Equipment, Multipliers, PhysicsFlag, SkillTree};
use protocol::packet::creature_update::Animation::*;
use protocol::packet::creature_update::CombatClassMajor::*;
use protocol::packet::creature_update::CombatClassMinor::*;
use protocol::packet::CreatureUpdate;
use protocol::utils::flagset::{FlagSet16, FlagSet32};

use crate::creature::Creature;

pub fn inspect_creature_update(packet: &CreatureUpdate, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	packet.position          .map_or_ok(|value| inspect_position(          value, &former_state, &updated_state))?;
	packet.rotation          .map_or_ok(|value| inspect_rotation(          value, &former_state, &updated_state))?;
	packet.velocity          .map_or_ok(|value| inspect_velocity(          value, &former_state, &updated_state))?;
	packet.acceleration      .map_or_ok(|value| inspect_acceleration(      value, &former_state, &updated_state))?;
	packet.velocity_extra    .map_or_ok(|value| inspect_velocity_extra(    value, &former_state, &updated_state))?;
	packet.head_tilt         .map_or_ok(|value| inspect_head_tilt(         value, &former_state, &updated_state))?;
	packet.flags_physics     .map_or_ok(|value| inspect_flags_physics(     value, &former_state, &updated_state))?;
	packet.affiliation       .map_or_ok(|value| inspect_affiliation(       value, &former_state, &updated_state))?;
	packet.race              .map_or_ok(|value| inspect_race(              value, &former_state, &updated_state))?;
	packet.animation         .map_or_ok(|value| inspect_animation(         value, &former_state, &updated_state))?;
	packet.animation_time    .map_or_ok(|value| inspect_animation_time(    value, &former_state, &updated_state))?;
	packet.combo             .map_or_ok(|value| inspect_combo(             value, &former_state, &updated_state))?;
	packet.hit_time_out      .map_or_ok(|value| inspect_hit_time_out(      value, &former_state, &updated_state))?;
	packet.appearance        .map_or_ok(|value| inspect_appearance(        value, &former_state, &updated_state))?;
	packet.flags             .map_or_ok(|value| inspect_flags(             value, &former_state, &updated_state))?;
	packet.effect_time_dodge .map_or_ok(|value| inspect_effect_time_dodge( value, &former_state, &updated_state))?;
	packet.effect_time_stun  .map_or_ok(|value| inspect_effect_time_stun(  value, &former_state, &updated_state))?;
	packet.effect_time_fear  .map_or_ok(|value| inspect_effect_time_fear(  value, &former_state, &updated_state))?;
	packet.effect_time_chill .map_or_ok(|value| inspect_effect_time_chill( value, &former_state, &updated_state))?;
	packet.effect_time_wind  .map_or_ok(|value| inspect_effect_time_wind(  value, &former_state, &updated_state))?;
	packet.show_patch_time   .map_or_ok(|value| inspect_show_patch_time(   value, &former_state, &updated_state))?;
	packet.combat_class_major.map_or_ok(|value| inspect_combat_class_major(value, &former_state, &updated_state))?;
	packet.combat_class_minor.map_or_ok(|value| inspect_combat_class_minor(value, &former_state, &updated_state))?;
	packet.mana_charge       .map_or_ok(|value| inspect_mana_charge(       value, &former_state, &updated_state))?;
	packet.unknown24         .map_or_ok(|value| inspect_unknown24(         value, &former_state, &updated_state))?;
	packet.unknown25         .map_or_ok(|value| inspect_unknown25(         value, &former_state, &updated_state))?;
	packet.aim_offset        .map_or_ok(|value| inspect_aim_offset(        value, &former_state, &updated_state))?;
	packet.health            .map_or_ok(|value| inspect_health(            value, &former_state, &updated_state))?;
	packet.mana              .map_or_ok(|value| inspect_mana(              value, &former_state, &updated_state))?;
	packet.blocking_gauge    .map_or_ok(|value| inspect_blocking_gauge(    value, &former_state, &updated_state))?;
	packet.multipliers       .map_or_ok(|value| inspect_multipliers(       value, &former_state, &updated_state))?;
	packet.unknown31         .map_or_ok(|value| inspect_unknown31(         value, &former_state, &updated_state))?;
	packet.unknown32         .map_or_ok(|value| inspect_unknown32(         value, &former_state, &updated_state))?;
	packet.level             .map_or_ok(|value| inspect_level(             value, &former_state, &updated_state))?;
	packet.experience        .map_or_ok(|value| inspect_experience(        value, &former_state, &updated_state))?;
	packet.master            .map_or_ok(|value| inspect_master(            value, &former_state, &updated_state))?;
	packet.unknown36         .map_or_ok(|value| inspect_unknown36(         value, &former_state, &updated_state))?;
	packet.power_base        .map_or_ok(|value| inspect_power_base(        value, &former_state, &updated_state))?;
	packet.unknown38         .map_or_ok(|value| inspect_unknown38(         value, &former_state, &updated_state))?;
	packet.home_zone         .map_or_ok(|value| inspect_home_zone(         value, &former_state, &updated_state))?;
	packet.home              .map_or_ok(|value| inspect_home(              value, &former_state, &updated_state))?;
	packet.zone_to_reveal    .map_or_ok(|value| inspect_zone_to_reveal(    value, &former_state, &updated_state))?;
	packet.unknown42         .map_or_ok(|value| inspect_unknown42(         value, &former_state, &updated_state))?;
	packet.consumable        .map_or_ok(|value| inspect_consumable(        value, &former_state, &updated_state))?;
	packet.equipment         .map_or_ok(|value| inspect_equipment(         value, &former_state, &updated_state))?;
	packet.name              .map_or_ok(|value| inspect_name(              value, &former_state, &updated_state))?;
	packet.skill_tree        .map_or_ok(|value| inspect_skill_tree(        value, &former_state, &updated_state))?;
	packet.mana_cubes        .map_or_ok(|value| inspect_mana_cubes(        value, &former_state, &updated_state))?;

	Ok(())
}

fn inspect_position(position: &Point3<i64>, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_rotation(rotation: &EulerAngles, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
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
		.ok_or("rotation.yaw wasn't finite")
}
fn inspect_velocity(velocity: &Vector3<f32>, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_acceleration(acceleration: &Vector3<f32>, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	//todo
//	val actualXY = sqrt(it.x.pow(2) + it.y.pow(2))
//	if (!current.flags[CreatureFlag.Gliding]) {
//		actualXY.expectIn(0f..=accelLimitXY, "acceleration.XY")
//	}
//	if (current.flagsPhysics[PhysicsFlag.Swimming]) {
//		it.z.expectIn(-80f..=80f, "acceleration.Z")
//	} else if (current.flags[CreatureFlag.Climbing]) {
//		it.z.expectIn(setOf(0f, -16f, 16f), "acceleration.Z")
//	} else {
//		it.z.expect(0f, "acceleration.Z")
//	}
	Ok(())
}
fn inspect_velocity_extra(velocity_extra: &Vector3<f32>, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	let (max_xy, max_z): (f32, f32) =
		match updated_state.combat_class_major {
			CombatClassMajor::Ranger => (35.0, 17.0),
			_                        => ( 0.1,  0.0)//0.1 because the game doesnt reset all the way to 0
		};

	velocity_extra.xy()
		.magnitude()
		.ensure_at_most(max_xy, "retreat_horizontal_speed")?;
	velocity_extra.z
		.ensure_within(&(0.0..=max_z), "")
}
fn inspect_head_tilt(head_tilt: &f32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	head_tilt
		.ensure_within(&(-32.5..=45.0), "head_tilt")//negative when attacking downwards
}
fn inspect_flags_physics(flags_physics: &FlagSet32<PhysicsFlag>, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {


	Ok(())
}
fn inspect_affiliation(affiliation: &Affiliation, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	affiliation
	 	.ensure_exact(&Affiliation::Player, "affiliation")
}
fn inspect_race(race: &Race, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	const PLAYABLE_RACES: [Race; 16] = [
		ElfMale,
		ElfFemale,
		HumanMale,
		HumanFemale,
		GoblinMale,
		GoblinFemale,
		LizardmanMale,
		LizardmanFemale,
		DwarfMale,
		DwarfFemale,
		OrcMale,
		OrcFemale,
		FrogmanMale,
		FrogmanFemale,
		UndeadMale,
		UndeadFemale
	];

	race.ensure_one_of(PLAYABLE_RACES.as_slice(), "")
}
fn inspect_animation(animation: &Animation, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	let class_specific =//TODO: const
		match updated_state.combat_class_major {
			CombatClassMajor::Warrior => vec![Smash, Cyclone],
			CombatClassMajor::Ranger  => vec![Kick],
			CombatClassMajor::Mage    => match updated_state.combat_class_minor {
				CombatClassMinor::Default     => vec![Teleport, FireExplosionShort],
				CombatClassMinor::Alternative => vec![Teleport, HealingStream],
				_ => unreachable!("invalid CombatClassMinor")
			}
			CombatClassMajor::Rogue   => match updated_state.combat_class_minor {
				CombatClassMinor::Default     => vec![Intercept, Stealth],
				CombatClassMinor::Alternative => vec![Intercept, Stealth, Shuriken],
				_ => unreachable!("invalid CombatClassMinor")
			}
			_ => unreachable!("invalid CombatClassMajor")
		};

	let mainhand = updated_state.equipment.right_weapon.type_minor;
	let offhand = updated_state.equipment.left_weapon.type_minor;

	//todo: need type_minor enums for this

	Ok(())
}
fn inspect_animation_time(animation_time: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
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
fn inspect_combo(combo: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	combo.ensure_not_negative("combo")
}
fn inspect_hit_time_out(hit_time_out: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
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
fn inspect_appearance(appearance: &Appearance, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	//appearance.flags //todo: all false
	//eex
	Ok(())
}
fn inspect_flags(flags: &FlagSet16<CreatureFlag>, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_effect_time_dodge(effect_time_dodge: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	effect_time_dodge.ensure_within(&(0..=600), "effect_time_dodge")
}
fn inspect_effect_time_stun(effect_time_stun: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	//todo: ensure positive when increased
	Ok(())
}
fn inspect_effect_time_fear(effect_time_fear: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	effect_time_fear.ensure_not_negative("effect_time_fear")
}
fn inspect_effect_time_chill(effect_time_chill: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	effect_time_chill.ensure_not_negative("effect_time_chill")
}
fn inspect_effect_time_wind(effect_time_wind: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	effect_time_wind.ensure_within(&(0..=5000), "effect_time_wind")
}
fn inspect_show_patch_time(show_patch_time: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_combat_class_major(combat_class_major: &CombatClassMajor, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	combat_class_major.ensure_one_of([Warrior, Ranger, Mage, Rogue].as_slice(), "combat_class_major")
	//todo: recheck gear
}
fn inspect_combat_class_minor(combat_class_minor: &CombatClassMinor, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	combat_class_minor.ensure_one_of([Default, Alternative].as_slice(), "combat_class_minor")
}
fn inspect_mana_charge(mana_charge: &f32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	mana_charge.ensure_at_most(updated_state.mana, "mana_charge")
}
fn inspect_unknown24(unknown24: &[f32; 3], former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_unknown25(unknown25: &[f32; 3], former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_aim_offset(aim_offset: &Point3<f32>, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	//aim_offset.magnitude().ensure_at_most(60.0, "aim_offset_distance") //todo: account for rounding errors and movement
	Ok(())
}
fn inspect_health(health: &f32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	//todo: calculate max hp
	Ok(())
}
fn inspect_mana(mana: &f32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
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
fn inspect_blocking_gauge(blocking_gauge: &f32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	const BLOCKING_ANIMATIONS: [Animation; 4] = [
		ShieldM2Charging,
		DualWieldM2Charging,
		GreatweaponM2Charging,
		UnarmedM2Charging //todo: can non-warriors block?
	];

	let max = if updated_state.animation.present_in(&BLOCKING_ANIMATIONS) {
		former_state.blocking_gauge
	} else {
		1.0
	};

	blocking_gauge.ensure_within(&(0.0..=max), "blocking_gauge") //todo: negative gauge glitch?
}
fn inspect_multipliers(multipliers: &Multipliers, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	//eex
	Ok(())
}
fn inspect_unknown31(unknown31: &i8, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_unknown32(unknown32: &i8, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_level(level: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	level.ensure_within(&(1..=500), "level")
}
fn inspect_experience(experience: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	let max = 9999;//todo: calc max xp based on lvl
	experience.ensure_within(&(0..=max), "experience")
}
fn inspect_master(master: &CreatureId, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	master
		.ensure_exact(&CreatureId(0), "master")
}
fn inspect_unknown36(unknown36: &i64, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_power_base(power_base: &i8, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	power_base
		.ensure_exact(&0, "power_base")
}
fn inspect_unknown38(unknown38: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_home_zone(home_zone: &Point3<i32>, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_home(home: &Point3<i64>, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_zone_to_reveal(zone_to_reveal: &Point3<i32>, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_unknown42(unknown42: &i8, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	Ok(())
}
fn inspect_consumable(consumable: &Item, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
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
fn inspect_equipment(equipment: &Equipment, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
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
fn inspect_name(name: &String, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	name.as_bytes().len().ensure_within(&(1..=15), "name.length")
	//todo: limit characters to what the default font can display
}
fn inspect_skill_tree(skill_tree: &SkillTree, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
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
fn inspect_mana_cubes(mana_cubes: &i32, former_state: &Creature, updated_state: &Creature) -> Result<(), &'static str> {
	mana_cubes.ensure_not_negative("mana_cubes")
}



trait EnsureNotNegative {
	fn ensure_not_negative<'a>(&self, property_name: &'a str) -> Result<(), &'a str>;
}
impl EnsureNotNegative for i32 {
	fn ensure_not_negative<'a>(&self, property_name: &'a str) -> Result<(), &'a str> {
		(!self.is_negative()).ok_or(property_name)
	}
}

trait EnsureAtMost: PartialOrd + Sized {
	fn ensure_at_most<'a>(&self, limit: Self, property_name: &'a str) -> Result<(), &'a str> {
		(*self <= limit).ok_or(property_name)
	}
}
impl<T: PartialOrd> EnsureAtMost for T {}

trait EnsureWithin: PartialOrd + Sized {
	fn ensure_within<'a>(&self, range: &impl RangeBounds<Self>, property_name: &'a str) -> Result<(), &'a str> {
		range
			.contains(&self)
			.ok_or(property_name)//format!("{} was {} instead of {}", property_name, self, container).as_str()
	}
}
impl<T: PartialOrd> EnsureWithin for T {}



trait EnsureOneOf: PartialEq + Sized {
	fn ensure_one_of<'a>(&self, range: &[Self], property_name: &'a str) -> Result<(), &'a str> {
		range
			.contains(self)
			.ok_or(property_name)//format!("{} was {} instead of {}", property_name, self, container).as_str()
	}

	fn ensure_exact<'a>(&self, expected: &Self, property_name: &'a str) -> Result<(), &'a str> {
		(self == expected)
			.ok_or(property_name)
	}
}
impl<T: PartialEq> EnsureOneOf for T {}



trait PresentIn: PartialEq + Sized {
	fn present_in(&self, container: &[Self]) -> bool {
		container.contains(self)
	}
}

impl<T: PartialEq> PresentIn for T {}



trait MapOrOk<T> {
	fn map_or_ok<E>(&self, f: impl FnOnce(&T) -> Result<(), E>) -> Result<(), E>;
}

impl<T> MapOrOk<T> for Option<T> {
	fn map_or_ok<E>(&self, f: impl FnOnce(&T) -> Result<(), E>) -> Result<(), E> {
		match self {
			Some(x) => f(x),
			Option::None => Ok(())
		}
	}
}