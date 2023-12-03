use std::str::SplitWhitespace;

use protocol::packet::common::Item;
use protocol::packet::common::item::Kind::*;
use protocol::packet::common::item::kind::Special::*;
use protocol::packet::common::item::kind::Weapon::*;
use protocol::packet::common::item::Material::*;
use protocol::packet::creature_update::Occupation;
use protocol::packet::world_update::Pickup;
use protocol::packet::WorldUpdate;
use protocol::utils::constants::rarity::LEGENDARY;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Gear;
use crate::addon::command_manager::utils::INGAME_ONLY;
use crate::server::player::Player;
use crate::server::Server;

impl Command for Gear {
	const LITERAL: &'static str = "gear";
	const ADMIN_ONLY: bool = false;

	async fn execute<'fut>(&'fut self, _server: &'fut Server, caller: Option<&'fut Player>, _params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
		let caller = caller.ok_or(INGAME_ONLY)?;
		let character = caller.character.read().await;

		let world_update = WorldUpdate {
			pickups: create_items(character.occupation, character.level as i16)
				.into_iter()
				.map(|item| Pickup {
					interactor: caller.id,
					item,
				}).collect(),
			..Default::default()
		};
		drop(character);

		caller.send_ignoring(&world_update).await;

		Ok(Option::None)
	}
}

fn create_items(occupation: Occupation, level: i16) -> Vec<Item> {
	let (armor_material, weapon_material, weapon_types) =
		match occupation {
			Occupation::Warrior => (Iron  , Iron, vec![Sword, Sword, Axe, Axe, Mace, Mace, Greatsword, Greataxe, Greatmace, Shield]),
			Occupation::Ranger  => (Linen , Wood, vec![Bow, Crossbow, Boomerang]),
			Occupation::Mage    => (Silk  , Wood, vec![Wand, Staff]),
			Occupation::Rogue   => (Cotton, Iron, vec![Longsword, Dagger, Dagger, Fist, Fist]),
			_ => unreachable!("player's occupation wasn't a combat discipline") //todo: err instead of panic?
		};

	let mut items = vec![
		Item {
			kind: Lamp,
			rarity: LEGENDARY,
			material: Iron,
			level: 1,
			..Default::default()
		},
		Item {
			kind: Special(HangGlider),
			material: Wood,
			level: 1,
			..Default::default()
		},
		Item {
			kind: Special(Boat),
			material: Wood,
			level: 1,
			..Default::default()
		},
		Item {
			kind: Coin,
			material: Gold,
			level: 1,
			..Default::default()
		}
	];

	weapon_types
		.into_iter()
		.map(|weapon_type| Item {
			kind: Weapon(weapon_type),
			rarity: LEGENDARY,
			material: weapon_material,
			level,
			..Default::default()
		})
		.collect_into(&mut items);

	[Chest, Gloves, Boots, Shoulder]
		.into_iter()
		.map(|kind| Item {
			kind,
			rarity: LEGENDARY,
			material: armor_material,
			level,
			..Default::default()
		})
		.collect_into(&mut items);

	let mut accessories = vec![Ring, Ring, Amulet];
	if occupation == Occupation::Mage {
		accessories.push(Weapon(Bracelet));
		accessories.push(Weapon(Bracelet));
	}
	accessories
		.into_iter()
		.flat_map(|kind| [Gold, Silver]
			.map(|material|
				Item {
					kind,
					rarity: LEGENDARY,
					material,
					level,
					..Default::default()
				})
		).collect_into(&mut items);

	items
}