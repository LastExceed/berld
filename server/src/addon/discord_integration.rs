use std::fs;

use twilight_gateway::Shard;
use twilight_http::Client;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::gateway::event::Event::MessageCreate;
use twilight_model::id::Id;

use protocol::packet::ChatMessageFromServer;
use protocol::packet::common::CreatureId;
use crate::addon::command_manager::CommandResult;

use crate::server::Server;

const PUBLIC_CHANNEL_ID: u64 = 1067011357129580667;
const ADMIN_CHANNEL_ID: u64 = 1088047136698011659;

#[derive(Debug)]
pub struct DiscordIntegration {
	http: Client,
	token: String
}

impl Default for DiscordIntegration {
	fn default() -> Self {
		let Ok(token) = fs::read_to_string(Self::FILE_PATH) else {
			fs::write(Self::FILE_PATH, "insert token here").unwrap();
			panic!("{} not found, created dummy file", Self::FILE_PATH);
		};

		Self {
			http: Client::new(token.clone()),
			token,
		}
	}
}

impl DiscordIntegration {
	const FILE_PATH: &'static str = "discord_bot_token.txt";

	pub fn run(&self, server: &Server) {
		let mut shard = Shard::new(
			ShardId::ONE,
			self.token.clone(),
			Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT
		);

		let server_static = server.extend_lifetime();

		tokio::spawn(async move {
			loop {
				match shard.next_event().await {
					Ok(MessageCreate(message)) if !message.author.bot => {
						let admin = match message.channel_id.get() {
							PUBLIC_CHANNEL_ID => false,
							ADMIN_CHANNEL_ID => true,
							_ => continue,
						};

						let callback = |response| { Self::command_callback(server_static, response, admin) };

						let is_command = server_static.addons.command_manager.on_message(
							server_static,
							None,
							admin,
							&message.content,
							'.',
							callback
						).await;

						if is_command {
							continue;
						}

						server_static.broadcast(&ChatMessageFromServer {//dont use server.announce() as that would cause an echo
							source: CreatureId(0),
							text: format!("<{}> {}", message.author.name, message.content)
						}, None).await;
					},

					Ok(_) => (),

					#[expect(clippy::dbg_macro, reason = "keeping this until i figure out the errors")]
					Err(error) => {
						dbg!(&error);//todo: proper error handling
						assert!(!error.is_fatal(), "fatal error in event loop of discord integration");
						continue;
					}
				};
			}
		});
	}

	pub async fn post(&self, message: &str, admin: bool) {
		self.http.create_message(Id::new(if admin { ADMIN_CHANNEL_ID } else { PUBLIC_CHANNEL_ID }))
			.content(message)
			.expect("setting content failed")
			.await
			.expect("create message failed");
	}

	async fn command_callback(server: &Server, result: CommandResult, admin: bool) {
		match result {
			Ok(response_option) => {
				let Some(response) = response_option
					else { return };

				server.addons.discord_integration.post(&response, admin).await;
			}
			Err(error) => {
				server.addons.discord_integration.post(error, admin).await;
			}
			// TODO: Remove duplicated code, by replacing &str with String
		}
	}
}