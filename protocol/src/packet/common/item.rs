use std::mem::{size_of, transmute};
use std::slice;

use nalgebra::Point3;
use strum_macros::{EnumCount, EnumIter};

use kind::*;

use crate::packet::common::{Item, Race};
use crate::packet::common::item::Kind::Formula;
use crate::utils::{ArrayWrapper, level_scaling_factor, rarity_scaling_factor};

pub mod kind;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum Kind {
	#[default]
	Void,
	Consumable(Consumable),
	Formula,
	Weapon(Weapon),
	Chest,
	Gloves,
	Boots,
	Shoulder,
	Amulet,
	Ring,
	Block,
	Resource(Resource),
	Coin,
	PlatinumCoin,
	Leftovers,
	Beak,
	Painting,
	Vase,
	Candle(Candle),
	Pet(Race),
	PetFood(Race),
	Quest(Quest),
	Unknown,
	Special(Special),
	Lamp,
	ManaCube
}

#[repr(i8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, EnumIter)]
pub enum Material {
	#[default]
	None,
	Iron,
	Wood,


	Obsidian = 5,
	Unknown,
	Bone,


	Copper = 10,
	Gold,
	Silver,
	Emerald,
	Sapphire,
	Ruby,
	Diamond,
	Sandstone,
	Saurian,
	Parrot,
	Mammoth,
	Plant,
	Ice,
	Licht,
	Glass,
	Silk,
	Linen,
	Cotton,

	Fire = i8::MIN,
	Unholy,
	IceSpirit,
	Wind,
}

#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ItemFlag {
	Adapted
}

impl From<ItemFlag> for u8 {
	fn from(it: ItemFlag) -> Self {
		it as Self
	}
}

#[repr(C, align(4))]
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Spirit {
	pub position: Point3<i8>,
	pub material: Material,
	pub level: i16,
	//pad2 //todo: struct align suggests that this could be a property, maybe seed/rarity/flags of the spirit?
}


///an awful workaround for the typesafety breaking inconsistency of formulas. initialize with Default::default(), and use the get/set functions afterwards
#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct RecipeDummy([u8;4]);

#[derive(Debug)]
pub struct NotAFormula;

impl Item {
	pub fn get_recipe(&self) -> Result<Kind, NotAFormula> {
		if self.kind != Formula {
			return Err(NotAFormula);
		}

		let recipe = unsafe {
			let recipe_bytes = [
				self._recipe.0[0],
				transmute::<_, [u8; size_of::<Kind>()]>(self.kind)[1]
			];

			transmute(recipe_bytes)
		};

		Ok(recipe)
	}

	pub fn set_recipe(&mut self, recipe: Kind) -> Result<(), NotAFormula> {
		if self.kind != Formula {
			return Err(NotAFormula);
		}

		unsafe {
			let recipe_bytes: [u8; size_of::<Kind>()] = transmute(recipe);

			slice::from_raw_parts_mut(
				(self as *mut Self).cast::<u8>(),
				size_of::<Self>()
			)[1] = recipe_bytes[1];

			self._recipe.0[0] = recipe_bytes[0];
		}

		Ok(())
	}
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, EnumIter, EnumCount)]
pub enum Stat {
	Damage,
	Armor,
	Resi,
	Health,
	Reg,
	Crit,
	Tempo
}

impl From<Stat> for usize {
	fn from(value: Stat) -> Self {
		value as Self
	}
}

type Stats = ArrayWrapper<Stat, f32>;

impl Item {
	pub fn stats(&self) -> Stats {
		use Kind::*;
		use kind::Weapon::*;
		use Material::*;
		//local imports are necessary as these enums are defined in this file

		let can_have_stat =//dmg, defense, hp/reg, crit/tempo
			match self.kind {
				Weapon(_) => (true, false, true, true),

				Chest     |
				Gloves    |
				Boots     |
				Shoulder  => (false, true, true, true),

				Ring      |
				Amulet    => (false, false, false, true),

				_         => (false, false, false, false)
			};

		let size_multiplier =
			match self.kind {
				Weapon(Bow)        |
				Weapon(Crossbow)   |
				Weapon(Boomerang)  |
				Weapon(Staff)      |
				Weapon(Wand)       |
				Weapon(Greatsword) |
				Weapon(Greataxe)   |
				Weapon(Greatmace)  |
				Weapon(Pitchfork)  |
				Chest              => 2.0,

				_                  => 1.0
			};

		let class_multiplier =
			match self.kind {
				Weapon(Longsword) |
				Weapon(Dagger)    |
				Weapon(Fist)      => 0.5,

				Weapon(Shield)    => 0.5,

				_                 => 1.0,
			};

		let material_multiplier =
			match self.material {
				Iron    => (1.0 , 0.85, 2.0 , 0.0, 0.0, 0.0),
				Linen   => (0.85, 0.75, 1.5 , 0.5, 0.0, 0.0),
				Cotton  => (0.85, 0.75, 1.75, 1.0, 0.0, 0.0),
				Silk    => (0.75, 1.0 , 1.0 , 0.0, 0.0, 0.0),
				Licht   => (0.75, 1.0 , 1.0 , 0.0, 0.0, 0.0),
				Parrot  => (0.85, 0.85, 1.0 , 0.0, 0.0, 0.0),
				Saurian => (0.8 , 1.0 , 1.0 , 0.0, 0.0, 0.0),
				Gold    => (1.0 , 1.0 , 1.0 , 0.0, 1.0, 0.0),
				Silver  => (1.0 , 1.0 , 1.0 , 0.0, 0.0, 1.0),
				_       => (1.0 , 1.0 , 1.0 , 0.0, 0.0, 0.0)
			};            //armor,resi,health,reg,crit,tempo

		let hp_reg_balance =    ((self.seed as u32 & 0x1FFFFFFF) * 8 % 21) as f32 / 20.0;
		let crit_tempo_balance = (self.seed as u32                   % 21) as f32 / 20.0;

		let spirit_bonus = self.spirit_counter as f32 * 0.1;

		[
			(4.0        , can_have_stat.0, false         , class_multiplier     , 0.0                     , true ),//dmg
			(0.5        , can_have_stat.1, false         , material_multiplier.0, 0.0                     , true ),//armor
			(0.5        , can_have_stat.1, false         , material_multiplier.1, 0.0                     , true ),//resi
			(2.5        , can_have_stat.2, true          , material_multiplier.2, 1.0 - hp_reg_balance    , true ),//hp
			(0.1        , can_have_stat.2, true          , material_multiplier.3, 0.0 + hp_reg_balance    , false),//reg
			(1.0 / 160.0, can_have_stat.3, false         , material_multiplier.4, 1.0 - crit_tempo_balance, false),//crit
			(1.0 /  80.0, can_have_stat.3, false         , material_multiplier.5, 0.0 + crit_tempo_balance, false) //tempo
		].map(|(base_value, stat_exists  , no_2h_doubling, material_multiplier  , seed_phase_bonus        , apply_spirit_bonus)|{
			if !stat_exists {
				return 0.0;
			}

			let skip_size = no_2h_doubling && matches!(self.kind, Weapon(_));

			base_value
				* if skip_size { 1.0 } else { size_multiplier }
				* (material_multiplier + seed_phase_bonus)
				* level_scaling_factor(self.level as f32 + if apply_spirit_bonus { spirit_bonus } else { 0.0 })
				* rarity_scaling_factor(self.rarity)
		}).into()
	}
}