#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Kind {
	Normal,
	Spark,

	NoSpreadNoRotation = 3,
	NoGravity
}