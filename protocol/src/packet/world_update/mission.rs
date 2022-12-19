#[repr(u8)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum MissionState {
	Ready,
	InProgress,
	Finished
}