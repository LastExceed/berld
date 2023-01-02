use crate::packet::common::Item;
use crate::packet::common::item::{TypeMajor, TypeMinor};
use crate::utils::constants::ItemType;

impl Item {
	pub fn item_type(&self) -> ItemType {
		ItemType {
			major: self.type_major,
			minor: self.type_minor
		}
	}
}

macro_rules! item_types {
    ($type_major:expr, $first:ident, $($rest:ident),*) => (
        item_types!($type_major, $($rest),+ ; 0; $first = 0);
    );
    ($type_major:expr, $current:ident, $($rest:ident),* ; $previous_index: expr ; $($item_name:ident = $index:expr)+) => (
        item_types!($type_major, $($rest),* ; $previous_index + 1; $($item_name = $index)* $current = $previous_index + 1);
    );
    ($type_major:expr, $last:ident; $previous_index:expr ; $($item_name:ident = $index:expr)+) => (
        $(pub const $item_name: ItemType = ItemType {
            major: $type_major,
            minor: TypeMinor($index)
        };)*
        pub const $last: ItemType = ItemType {
            major: $type_major,
            minor: TypeMinor($previous_index + 1)
        };
    );
}

pub const VOID: ItemType = ItemType {
	major: TypeMajor::Void,
	minor: TypeMinor(0),
};

item_types!(TypeMajor::Consumable,
	COOKIE,
	LIFE_POTION,
	CACTUS_POTION,
	MANA_POTION,
	GINSENG_SOUP,
	SNOWBERRY_MASH,
	MUSHROOM_SPIT,
	BOMB,
	PINEAPPLE_SLICE,
	PUMPKIN_MUFFIN
);

item_types!(TypeMajor::Weapon,
	SWORD,
	AXE,
	MACE,
	DAGGER,
	FIST,
	LONGSWORD,
	BOW,
	CROSSBOW,
	BOOMERANG,
	ARROW,
	STAFF,
	WAND,
	BRACELET,
	SHIELD,
	QUIVER,
	GREATSWORD,
	GREATAXE,
	GREATMACE,
	PITCHFORK,
	PICKAXE,
	TORCH
);

item_types!(TypeMajor::Resource,
	NUGGET,
	LOG,
	FEATHER,
	HORN,
	CLAW,
	FIBER,
	COBWEB,
	HAIR,
	CRYSTAL,
	YARN,
	CUBE,
	CAPSULE,
	FLASK,
	ORB,
	SPIRIT,
	MUSHROOM,
	PUMPKIN,
	PINEAPPLE,
	RADISHSLICE,
	SHIMMERMUSHROOM,
	GINSENGROOT,
	ONIONSLICE,
	HEARTFLOWER,
	PRICKLYPEAR,
	ICEFLOWER,
	SOULFLOWER,
	WATERFLASK,
	SNOWBERRY
);

item_types!(TypeMajor::Candle,
	RED_CANDLE,
	GREEN_CANDLE
);

item_types!(TypeMajor::Quest,
	AMULETYELLOW,
	AMULETBLUE,
	JEWELCASE,
	KEY,
	MEDICINE,
	ANITVENOM,
	BANDAID,
	CRUTCH,
	BANDAGE,
	SALVE
);

item_types!(TypeMajor::Special,
	HANG_GLIDER,
	BOAT
);