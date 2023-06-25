#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Kind {
	Bomb = 1,
	Talk,
	ObjectInteraction,

	PickUp = 5,
	Drop,

	CallPet = 8
}