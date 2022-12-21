#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProjectileType {
	Arrow,
	Magic,
	Boomerang,
	Unknown,
	Boulder
}