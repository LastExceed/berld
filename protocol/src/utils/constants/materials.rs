use crate::packet::common::item::{self, Material};
use crate::packet::common::item::Material::*;
use crate::packet::creature_update::Occupation;

#[must_use]
pub const fn by_item_kind(item_kind: item::Kind) -> &'static [Material] {
	use item::Kind::*;
	use item::kind::Weapon::*;
	
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

		Chest|Boots|Gloves|Shoulder            => &[Bone, Mammoth, Gold,
		                                            Iron, Obsidian, Saurian, Ice,
													Parrot, Linen,
													Licht, Silk,
													Cotton            ][..],

		Amulet|Ring                            => &[Gold,Silver][..],

		Special(_)                             => &[Wood][..],
		Lamp                                   => &[Iron][..],
		_                                      => &[None][..],
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
