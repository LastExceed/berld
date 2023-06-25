#[repr(u32)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Kind {
	Arrow,
	Magic,
	Boomerang,
	Unknown,
	Boulder
}