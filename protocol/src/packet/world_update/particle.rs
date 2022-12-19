#[repr(i32)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum ParticleType {
	Normal,
	Spark,

	NoSpreadNoRotation = 3,
	NoGravity
}