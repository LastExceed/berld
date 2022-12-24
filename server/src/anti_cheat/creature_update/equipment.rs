use protocol::packet::common::item::Material;
use protocol::packet::common::item::Material::*;
use protocol::packet::common::item::TypeMajor::*;
use protocol::packet::creature_update::CombatClassMajor;
use protocol::packet::creature_update::CombatClassMajor::*;
use protocol::utils::constants::{ItemType, materials};
use protocol::utils::constants::item_types::*;

pub(super) fn allowed_materials(item_type: ItemType, combat_class_major: CombatClassMajor) -> &'static [Material] {
	match item_type.major {
		Weapon => match item_type {
			SWORD      => &materials::SWORD[..],
			AXE        => &materials::AXE[..],
			MACE       => &materials::MACE[..],
			DAGGER     => &materials::DAGGER[..],
			FIST       => &materials::FIST[..],
			LONGSWORD  => &materials::LONGSWORD[..],
			BOW        => &materials::BOW[..],
			CROSSBOW   => &materials::CROSSBOW[..],
			BOOMERANG  => &materials::BOOMERANG[..],
			ARROW      => &materials::ARROW[..],
			STAFF      => &materials::STAFF[..],
			WAND       => &materials::WAND[..],
			BRACELET   => &materials::BRACELET[..],
			SHIELD     => &materials::SHIELD[..],
			QUIVER     => &materials::QUIVER[..],
			GREATSWORD => &materials::GREATSWORD[..],
			GREATAXE   => &materials::GREATAXE[..],
			GREATMACE  => &materials::GREATMACE[..],
			PITCHFORK  => &materials::PITCHFORK[..],
			PICKAXE    => &materials::PICKAXE[..],
			TORCH      => &materials::TORCH[..],
			_          => &[Material::None][..]
		},

		Chest |
		Boots |
		Gloves |
		Shoulder => match combat_class_major {
			Warrior => &[Bone, Mammoth, Gold, Iron, Obsidian, Saurian, Ice][..],
			Ranger  => &[Bone, Mammoth, Gold, Parrot, Linen][..],
			Mage    => &[Bone, Mammoth, Gold, Licht, Silk][..],
			Rogue   => &[Bone, Mammoth, Gold, Cotton][..],
			_       => &[Bone, Mammoth, Gold][..]
		},//todo: burn it with fire

		Amulet |
		Ring    => &materials::ACCESSORIES[..],

		Special => &materials::SPECIAL[..],

		Lamp    => &materials::LAMP[..],

		Pet |
		PetFood |
		_ => &[Material::None][..],
	}
}