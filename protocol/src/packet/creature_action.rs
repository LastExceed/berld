#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Kind {
	Bomb = 1,
	Talk,
	ObjectInteraction,

	PickUp = 5,
	Drop,

	CallPet = 8
}