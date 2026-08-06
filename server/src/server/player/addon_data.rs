use std::collections::HashMap;
use std::time::{Duration, Instant};

use protocol::nalgebra::Point2;

use crate::addon::anti_cheat::PlayerData;

// zone update safety valve
pub const AWAITED_GRACE: Duration = Duration::from_secs(30);

#[derive(Debug, Default)]
pub struct AddonData {
	pub team: Option<i32>,
	pub anti_cheat_data: PlayerData,
	pub last_attacker: Option<(Instant, String)>,
	pub zone_states: HashMap<Point2<i32>, ZoneState>,
	pub revealing_neighborhood: bool
}

#[derive(Debug, Clone, Copy)]
pub enum ZoneState {
	Awaited(Instant),
	Pending,
	Revealed(Instant)
}