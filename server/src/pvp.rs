use protocol::packet::creature_update::CreatureUpdate;

const FRIENDLY_FIRE_FLAG: u16 = 1 << 5;

pub fn enable_pvp(creature_update: &mut CreatureUpdate) {
	if let Some(flags) = creature_update.flags {
		creature_update.flags = Some(flags | FRIENDLY_FIRE_FLAG)
	}
}