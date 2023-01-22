use protocol::packet::{StatusEffect, WorldUpdate};
use protocol::packet::status_effect::StatusEffectType::Swiftness;

use crate::server::Server;

pub async fn buff_warfrenzy(warfrenzy: &StatusEffect, server: &Server) {
	let swiftness = StatusEffect {
		type_: Swiftness,
		..*warfrenzy
	};
	// sending this separately from the original status effect
	// as that one isn't sent back to the source
	server.broadcast(&WorldUpdate::from(swiftness), None).await;
}