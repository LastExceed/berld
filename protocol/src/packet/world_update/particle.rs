#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
	Normal,
	Spark,

	NoSpreadNoRotation = 3,
	NoGravity
}