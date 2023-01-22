use protocol::packet::creature_update::CreatureFlag;
use protocol::packet::CreatureUpdate;

use crate::creature::Creature;

pub mod anti_cheat;
pub mod traffic_filter;
pub mod balancing;

pub fn enable_pvp(creature_update: &mut CreatureUpdate) {
	if let Some(ref mut flags) = creature_update.flags {
		flags.set(CreatureFlag::FriendlyFire, true)
	}
}

pub fn fix_cutoff_animations(creature_update: &mut CreatureUpdate, previous_state: &Creature) {
	if let Some(animation_time) = creature_update.animation_time && animation_time <= previous_state.animation_time {
		creature_update.animation_time = Some(0); //starts all animations from the beginning to prevent cut-off animations, at the cost of some minimal delay
	}
}