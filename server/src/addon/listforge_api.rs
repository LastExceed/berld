use std::env::consts::OS;
use std::net::Ipv6Addr;
use std::time::{Instant, Duration};

use axum::{Router, Json};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::extract::State;
use config::{Config, ConfigError};
use futures::future::join_all;
use serde::Serialize;
use tap::Pipe;
use tokio::net::TcpListener;

use crate::server::Server;
use crate::server::utils::extend_lifetime;

pub struct ListforgeApi {
	slots: i32,
	name: String,
	discord: String,
	port: u16
}

impl ListforgeApi {
	pub fn new(config: &Config) -> Result<Self, ConfigError> {
		Self {
			slots: config.get("slots")?,
			name: config.get("name")?,
			discord: config.get("discord_invite")?,
			port: config.get("listforgeapi_port")?
		}.pipe(Ok)
	}

	pub async fn run(&self, server: &Server) {
		let state = (
			extend_lifetime(server),
			Instant::now(),
			self.slots,
			self.name.clone(),
			self.discord.clone()
		);

		let router = Router::new()
			.route("/api/info", get(info))
			.with_state(state);

		let listener = TcpListener::bind((Ipv6Addr::UNSPECIFIED, self.port))
			.await
			.expect("failed to bind API socket");

		tokio::spawn(async move {
			axum::serve(listener, router)
			.await
			.expect("API error");
		});
	}
}

async fn info(State((server, startup_time, slots, name, discord)): State<(&Server, Instant, i32, String, String)>) -> impl IntoResponse {
	Info {
		players: get_all_player_names(server).await,
		platform: OS.into(),
		mapseed: server.mapseed,
		uptime: startup_time.elapsed(),
		slots,
		name,
		discord,
	}.pipe(Json)
}

#[derive(Serialize)]
struct Info {
	players: Vec<String>,
	platform: String,
	mapseed: i32,
	uptime: Duration,
	slots: i32,
	name: String,
	discord: String
}

async fn get_all_player_names(server: &Server) -> Vec<String> {
	server
		.players
		.read()
		.await
		.iter()
		.map(|player| async {
			player.character
				.read()
				.await
				.name
				.clone()
		})
		.pipe(join_all)
		.await
}