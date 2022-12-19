#[repr(u32)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum ProjectileType {
	Arrow,
	Magic,
	Boomerang,
	Unknown,
	Boulder
}