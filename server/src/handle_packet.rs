use std::io;

use protocol::packet::FromClient;

use crate::player::Player;

mod creature_update;
mod creature_action;
mod hit;
mod status_effect;
mod projectile;
mod chat_message;
mod zone_discovery;
mod region_discovery;

pub trait HandlePacket<Packet: FromClient> {
	fn handle_packet(&self, source: &Player, packet: Packet) -> Result<(), io::Error>;
}