use crate::packet::common::item::{self, Material};
use crate::packet::common::item::Material::*;
use crate::packet::creature_update::Occupation;

#[must_use]
pub const fn by_item_kind(item_kind: item::Kind) -> &'static [Material] {
	use item::Kind::*;
	use item::kind::Weapon::*;
	use item::kind::Resource::*;
	
	#[expect(clippy::match_same_arms, reason = "happenstance")]
	match item_kind {
		Weapon(Sword)                          => &[Iron,Obsidian,Bone][..],
		Weapon(Axe|Mace)                       => &[Iron,Bone         ][..],
		Weapon(Dagger|Fist|Longsword) 		   => &[Iron              ][..],
		Weapon(Bow|Crossbow|Boomerang)         => &[Wood              ][..],
		Weapon(Arrow)                          => &[None,Wood         ][..],
		Weapon(Staff)                          => &[Wood,Obsidian     ][..],
		Weapon(Wand)                           => &[Wood              ][..],
		Weapon(Bracelet)                       => &[Gold,Silver       ][..],
		Weapon(Shield)                         => &[Iron,Wood         ][..],
		Weapon(Quiver)                         => &[None              ][..],
		Weapon(Greatsword)                     => &[Iron              ][..],
		Weapon(Greataxe)                       => &[Iron,Saurian      ][..],
		Weapon(Greatmace)                      => &[Iron,Wood,Bone    ][..],
		Weapon(Pitchfork|Pickaxe)              => &[None,Iron,Wood    ][..],
		Weapon(Torch)                          => &[None,Wood         ][..],

		Chest|Boots|Gloves|Shoulder            => &[Iron,Obsidian,Bone,
													Gold,Saurian,Parrot,
													Mammoth,Ice,Licht,
													Silk,Linen,Cotton ][..],

		Amulet|Ring                            => &[Gold,Silver		  ][..],

		Resource(Nugget)					   => &[Iron,Wood,Gold,Silver,
													Emerald,Sapphire,Ruby,
													Diamond,Sandstone ][..],
		Resource(Log)						   => &[Wood			  ][..],
		Resource(Feather)					   => &[Parrot			  ][..],
		Resource(Fiber)						   => &[Plant			  ][..],
		Resource(Yarn)						   => &[Silk,Cotton		  ][..],
		Resource(Cube)                         => &[Iron,Wood,Obsidian,
											   		Bone,Gold,Silver  ][..],
		Resource(Capsule)					   => &[Cotton			  ][..],
		Resource(Flask)						   => &[Glass			  ][..],
		Resource(Spirit)					   => &[Fire,Unholy,
													IceSpirit,Wind	  ][..],

		Coin								   => &[Copper,Gold,Silver][..],
		Void|Block|Unknown|
		Resource(Horn|Claw|Crystal)			   => &[				  ][..],

		Special(_)                             => &[Wood			  ][..],
		Lamp                                   => &[Iron			  ][..],
		_                                      => &[None			  ][..],
	}
}

#[must_use]
pub const fn armour_exclusivity(material: Material) -> Option<Occupation> {
	use Occupation::*;
	match material {
		Iron|Obsidian|Saurian|Ice => Some(Warrior),
		Parrot|Linen              => Some(Ranger),
		Licht|Silk                => Some(Mage),
		Cotton                    => Some(Rogue),
		_                         => Option::None
	}
}