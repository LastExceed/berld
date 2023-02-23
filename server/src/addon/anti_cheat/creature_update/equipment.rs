use protocol::packet::common::item;
use protocol::packet::common::item::Kind::*;
use protocol::packet::common::item::kind::Weapon::*;
use protocol::packet::common::item::Material;
use protocol::packet::common::item::Material::*;
use protocol::packet::creature_update::Occupation;
use protocol::packet::creature_update::Occupation::*;
use protocol::utils::constants::materials;

pub(crate) fn allowed_materials(item_kind: item::Kind, occupation: Occupation) -> &'static [Material] {
	match item_kind {
		Weapon(Sword)      => &materials::SWORD[..],
		Weapon(Axe)        => &materials::AXE[..],
		Weapon(Mace)       => &materials::MACE[..],
		Weapon(Dagger)     => &materials::DAGGER[..],
		Weapon(Fist)       => &materials::FIST[..],
		Weapon(Longsword)  => &materials::LONGSWORD[..],
		Weapon(Bow)        => &materials::BOW[..],
		Weapon(Crossbow)   => &materials::CROSSBOW[..],
		Weapon(Boomerang)  => &materials::BOOMERANG[..],
		Weapon(Arrow)      => &materials::ARROW[..],
		Weapon(Staff)      => &materials::STAFF[..],
		Weapon(Wand)       => &materials::WAND[..],
		Weapon(Bracelet)   => &materials::BRACELET[..],
		Weapon(Shield)     => &materials::SHIELD[..],
		Weapon(Quiver)     => &materials::QUIVER[..],
		Weapon(Greatsword) => &materials::GREATSWORD[..],
		Weapon(Greataxe)   => &materials::GREATAXE[..],
		Weapon(Greatmace)  => &materials::GREATMACE[..],
		Weapon(Pitchfork)  => &materials::PITCHFORK[..],
		Weapon(Pickaxe)    => &materials::PICKAXE[..],
		Weapon(Torch)      => &materials::TORCH[..],

		Chest    |
		Boots    |
		Gloves   |
		Shoulder => match occupation {
			Warrior => &[Bone, Mammoth, Gold, Iron, Obsidian, Saurian, Ice][..],
			Ranger  => &[Bone, Mammoth, Gold, Parrot, Linen][..],
			Mage    => &[Bone, Mammoth, Gold, Licht, Silk][..],
			Rogue   => &[Bone, Mammoth, Gold, Cotton][..],
			_       => &[Bone, Mammoth, Gold][..]
		},//todo: burn it with fire

		Amulet     |
		Ring       => &materials::ACCESSORIES[..],

		Special(_) => &materials::SPECIAL[..],

		Lamp       => &materials::LAMP[..],

		Pet(_)     |
		PetFood(_) |
		_          => &[Material::None][..],
	}
}