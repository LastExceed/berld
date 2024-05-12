use std::time::Duration;

use config::{Config, ConfigError};
use futures::future::join_all;
use tap::Pipe;
use tokio::time::sleep;

use protocol::packet::{CreatureUpdate, IngameDatetime, WorldUpdate};
use protocol::packet::world_update::{Sound, sound};
use protocol::utils::sound_position_of;
use protocol::packet::world_update::sound::Kind::{MenuOpen2, MenuClose2};

use crate::addon::balancing::Balancing;
use crate::server::utils::extend_lifetime;
use crate::addon::command_manager::CommandManager;
use crate::addon::discord_integration::DiscordIntegration;
use crate::server::creature::Creature;
use crate::server::player::Player;
use crate::server::Server;

use listforge_api::ListforgeApi;

use self::models::Models;

pub mod anti_cheat;
pub mod traffic_filter;
pub mod balancing;
pub mod discord_integration;
pub mod command_manager;
pub mod pvp;
pub mod listforge_api;
pub mod kill_feed;
pub mod models;

pub struct Addons {
	pub discord_integration: DiscordIntegration,
	pub balancing: Balancing,
	pub command_manager: CommandManager,
	pub listforge_api: ListforgeApi,
	pub models: Models
}

impl Addons {
	pub fn new(config: &Config) -> Result<Self, ConfigError> {
		let instance = Self {
			discord_integration: DiscordIntegration::new(config)?,
			balancing: Balancing::new(config)?,
			command_manager: CommandManager::new(config)?,
			listforge_api: ListforgeApi::new(config)?,
			models: Models::new(config)?
		};

		Ok(instance)
	}
}

pub async fn announce_join_leave(server: &Server, player: &Player, joined: bool) {
	let name = &player.character.read().await.name;
	let sign = if joined { '+' } else { '-' };
	let sound = if joined { MenuOpen2 } else { MenuClose2 };

	server.announce(format!("[{sign}] {name}")).await;
	play_sound_for_everyone(server, sound, 2.0, 1.0).await;
}

pub fn fix_cutoff_animations(creature_update: &mut CreatureUpdate, previous_state: &Creature) {
	if let Some(ref mut animation_time) = creature_update.animation_time && *animation_time <= previous_state.animation_time {
		*animation_time = 0; //starts all animations from the beginning to prevent cut-off animations, at the cost of some minimal delay
	}
}

pub fn freeze_time(server: &Server) {
	let server_static = extend_lifetime(server);

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