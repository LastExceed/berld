use protocol::packet::FromClient;

use crate::server::player::Player;

mod creature_update;
mod creature_action;
mod hit;
mod status_effect;
mod projectile;
mod chat_message;
mod zone_request;
mod region_request;

pub trait HandlePacket<Packet: FromClient> {
	async fn handle_packet(&self, source: &Player, packet: Packet);
}