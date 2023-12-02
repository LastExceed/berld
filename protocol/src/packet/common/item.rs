use std::mem::transmute;

use nalgebra::Point3;
use strum_macros::{EnumCount, EnumDiscriminants, EnumIter};
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use kind::*;

use crate::{ReadCwData, Validate, Validator, WriteCwData};
use crate::packet::common::{Item, Race};
use crate::utils::{ArrayWrapper, level_scaling_factor, rarity_scaling_factor};
use crate::utils::io_extensions::{ReadArbitrary, WriteArbitrary};

pub mod kind;

impl Validate<Item> for Validator {
	fn validate(item: &Item) -> io::Result<()> {
		match item.kind {
			Kind::Void                   => Ok(()),
			Kind::Consumable(consumable) => Self::validate_enum(&consumable),
			Kind::Weapon(weapon)         => Self::validate_enum(&weapon),
			Kind::Resource(resource)     => Self::validate_enum(&resource),
			Kind::Candle(candle)         => Self::validate_enum(&candle),
			Kind::Pet(race) |
			Kind::PetFood(race)          => Self::validate_enum(&race),
			Kind::Quest(quest)           => Self::validate_enum(&quest),
			Kind::Special(special)       => Self::validate_enum(&special),
			_                            => Self::validate_enum(&item.kind)
		}?;
		Self::validate_enum(&item.material)
	}
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, EnumIter, EnumDiscriminants)]
pub enum Kind {
	#[default]
	Void,
	Consumable(Consumable),
	//Formula, //use item.as_formula instead //todo: recursive formulas
	Weapon(Weapon) = 3,
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, EnumIter)]
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
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Flag {
	Adapted
}

impl From<Flag> for usize {
	fn from(it: Flag) -> Self {
		it as Self
	}
}

#[repr(C, align(4))]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default)]
pub struct Spirit {
	pub position: Point3<i8>,
	pub material: Material,
	pub level: i16,
	//pad2 //todo: struct align suggests that this could be a property, maybe seed/rarity/flags of the spirit?
}

//custom read/write impl is necessary solely because of formula weirdness :(
impl<Writable: AsyncWrite + Unpin> WriteCwData<Item> for Writable {
	async fn write_cw_data(&mut self, item: &Item) -> io::Result<()> {
		let has_subkind = matches!(
			item.kind,
			Kind::Consumable(_)
			| Kind::Weapon(_)
			| Kind::Resource(_)
			| Kind::Candle(_)
			| Kind::Pet(_)
			| Kind::PetFood(_)
			| Kind::Quest(_)
			| Kind::Special(_)
		);
		//SAFETY: infallible
		let kind_bytes: [u8; 2] = unsafe { transmute(item.kind) };
		self.write_u8(if item.as_formula { 2 } else { kind_bytes[0] }).await?;
		self.write_u8(if has_subkind { kind_bytes[1] } else { 0 }).await?;
		self.write_all(&[0_u8; 2]).await?; //pad2
		self.write_i32_le(item.seed).await?;
		self.write_u32_le(if item.as_formula { kind_bytes[0] as _ } else { 0 }).await?;
		self.write_u8(item.rarity).await?;
		self.write_u8(item.material as _).await?;
		self.write_arbitrary(&item.flags).await?;
		self.write_all(&[0_u8; 1]).await?; //pad2
		self.write_i16_le(item.level).await?;
		self.write_all(&[0_u8; 2]).await?; //pad2
		self.write_arbitrary(&item.spirits).await?;
		self.write_i32_le(item.spirit_counter).await
	}
}

impl<Readable: AsyncRead + Unpin> ReadCwData<Item> for Readable {
	async fn read_cw_data(&mut self) -> io::Result<Item> {
		let mut mainkind = self.read_u8().await?;
		let subkind = self.read_u8().await?;
		let _ = self.read_u16().await?;
		let seed = self.read_i32_le().await?;
		let recipe = self.read_u32_le().await?;
		let rarity = self.read_u8().await?;
		let material = self.read_arbitrary().await?;
		let flags = self.read_arbitrary().await?;
		let _ = self.read_u8().await?;
		let level = self.read_i16_le().await?;
		let _ = self.read_u16().await?;

		let is_formula = mainkind == 2;
		if is_formula {
			mainkind = recipe as _;
		}
		//SAFETY: this value gets validated below
		let kind = unsafe { transmute([mainkind, subkind]) };

		let item = Item {
			kind,
			as_formula: is_formula,
			seed,
			rarity,
			material,
			flags,
			level,
			spirits: self.read_arbitrary().await?,
			spirit_counter: self.read_i32_le().await?,
		};
		Validator::validate(&item)?;
		Ok(item)
	}
}


#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumIter, EnumCount)]
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
	#[must_use]
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

		#[expect(clippy::match_same_arms, reason="coincidence")]
		let class_multiplier =
			match self.kind {
				Weapon(Longsword) |
				Weapon(Dagger)    |
				Weapon(Fist)      => 0.5,

				Weapon(Shield)    => 0.5,

				_                 => 1.0,
			};

		#[expect(clippy::match_same_arms, reason = "coincidence")]
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

		let hp_reg_balance =    ((self.seed as u32 & 0x1FFF_FFFF) * 8 % 21) as f32 / 20.0;
		let crit_tempo_balance = (self.seed as u32                    % 21) as f32 / 20.0;

		let spirit_bonus = self.spirit_counter as f32 * 0.1;

		#[expect(clippy::shadow_unrelated, reason="kinda false positive")]
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