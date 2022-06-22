#[repr(u8)]
pub enum CreatureActionType {
	Bomb = 1,
	Talk,
	ObjectInteraction,

	PickUp = 5,
	Drop,

	CallPet = 8
}