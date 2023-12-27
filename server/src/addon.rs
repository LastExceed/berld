use std::time::Duration;

use futures::future::join_all;
use tap::Pipe;
use tokio::time::sleep;

use protocol::packet::{CreatureUpdate, IngameDatetime, WorldUpdate};
use protocol::packet::world_update::{Sound, sound};
use protocol::packet::world_update::sound::Kind::MenuSelect;
use protocol::utils::sound_position_of;

use crate::addon::balancing::AirTimeTracker;
use crate::addon::command_manager::CommandManager;
use crate::addon::discord_integration::DiscordIntegration;
use crate::server::creature::Creature;
use crate::server::player::Player;
use crate::server::Server;

pub mod anti_cheat;
pub mod traffic_filter;
pub mod balancing;
pub mod discord_integration;
pub mod command_manager;
pub mod pvp;

#[derive(Default)]
pub struct Addons {
	pub discord_integration: DiscordIntegration,
	pub air_time_tracker: AirTimeTracker,
	pub command_manager: CommandManager
}

pub fn fix_cutoff_animations(creature_update: &mut CreatureUpdate, previous_state: &Creature) {
	if let Some(ref mut animation_time) = creature_update.animation_time && *animation_time <= previous_state.animation_time {
		*animation_time = 0; //starts all animations from the beginning to prevent cut-off animations, at the cost of some minimal delay
	}
}

pub fn freeze_time(server: &Server) {
	let server_static = server.extend_lifetime();

	tokio::spawn(async move {
		loop {
			server_static.broadcast(&IngameDatetime { time: 12 * 60 * 60 * 1000, day: 0 }, None).await;
			sleep(Duration::from_secs(6)).await;
		}
	});
}

pub async fn play_sound_for_everyone(server: &Server, kind: sound::Kind, pitch: f32, volume: f32) {
	//cant use broadcast as sound position is different for each player
	server.players
		.read()
		.await
		.iter()
		.map(|player| play_sound_at_player(player, kind, pitch, volume))
		.pipe(join_all)
		.await;
}

pub async fn play_sound_at_player(player: &Player, kind: sound::Kind, pitch: f32, volume: f32) {
	let sound = Sound {
		position: sound_position_of(player.character.read().await.position),
		kind,
		pitch,
		volume,
	};
	player.send_ignoring(&WorldUpdate::from(sound)).await;
}