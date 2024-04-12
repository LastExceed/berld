use protocol::packet::projectile::Kind::*;
use protocol::packet::world_update::Sound;
use protocol::utils::sound_position_of;
use protocol::packet::{Projectile, WorldUpdate};
use protocol::packet::world_update::sound;
use protocol::utils::constants::combat_classes::WATER_MAGE;
use rand::random;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<Projectile> for Server {
	async fn handle_packet(&self, source: &Player, packet: Projectile) {
		let mut world_update = WorldUpdate::from(packet.clone()); //todo: this clone should be avoidable

		if let Some(sound) = get_sound(source, packet).await {
			world_update.sounds.push(sound);
		}

		self.broadcast(&world_update, Some(source)).await;
	}
}

async fn get_sound(source: &Player, packet: Projectile) -> Option<Sound> {
	let character = source.character.read().await;

	#[allow(clippy::match_same_arms, reason = "comments")]
	match packet.kind {
		Arrow if packet.is_yellow
			=> Some((sound::Kind::Salvo2, 1.25, true)),
		Arrow
			=> Some((sound::Kind::Swing, 2.0, false)),
		Magic if character.combat_class() == WATER_MAGE
			=> Some((sound::Kind::Watersplash, 1.25, false)),
		Magic
			=> None, //probably [Fireball], but this sound is broken (cannot be heard)
		Boomerang
			=> Some((sound::Kind::Swing, 1.0, true)),
		Unknown
			=> None,
		Boulder
			=> None, //silent when used by a player, but maybe rockings make sound?
	}.map(|(kind, mut pitch, randomize_pitch)| {
		if randomize_pitch {
			let variance = random::<f32>() * 0.5 - 0.25;
			pitch += variance;
		}

		Sound {
			position: sound_position_of(character.position),
			kind,
			pitch,
			volume: 1.0,
		}
	})
}