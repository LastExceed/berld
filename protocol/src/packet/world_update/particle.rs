#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ParticleType {
	Normal,
	Spark,

	NoSpreadNoRotation = 3,
	NoGravity
}