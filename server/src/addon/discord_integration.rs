use config::{Config, ConfigError};
use tap::Pipe;
use twilight_gateway::{EventTypeFlags, Shard, StreamExt};
use twilight_http::Client;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::gateway::event::Event::MessageCreate;
use twilight_model::id::Id;

use protocol::packet::ChatMessageFromServer;
use protocol::packet::common::CreatureId;
use crate::{addon::command_manager::CommandResult, server::utils::log_error};
use crate::server::utils::extend_lifetime;

use crate::server::Server;

#[derive(Debug)]
pub struct DiscordIntegration {
	http: Client,
	token: String,
	public_channel: u64,
	admin_channel: u64
}

impl DiscordIntegration {
	pub fn new(config: &Config) -> Result<Self, ConfigError> {
		let token = config.get_string("discord_bot_token")?;

		let instance = Self {
			http: Client::new(token.clone()),
			token,
			public_channel: config.get("discord_public_channel_id")?,
			admin_channel: config.get("discord_admin_channel_id")?,
		};

		Ok(instance)
	}
}

impl DiscordIntegration {
	pub fn run(&self, server: &Server) {
		let mut shard = Shard::new(
			ShardId::ONE,
			self.token.clone(),
			Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT
		);

		let server_static = extend_lifetime(server);
		let self_static = extend_lifetime(self);

		tokio::spawn(async move {
			loop {
				match shard.next_event(EventTypeFlags::MESSAGE_CREATE).await.expect("shard stream ended") {
					Ok(MessageCreate(message)) if message.author.bot => {} // ignore

					Ok(MessageCreate(message)) => {
						let channel_id = message.channel_id.get();

						let admin = if channel_id == self_static.admin_channel {
							true
						} else if channel_id == self_static.public_channel {
							false
						} else {
							continue
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

					Ok(event) => { dbg!("unexpected discord event", event); },

					#[expect(clippy::dbg_macro, reason = "keeping this until i figure out the errors")]
					Err(error) => { dbg!(&error); } // todo: proper error handling
				}
			}
		});
	}

	pub async fn post(&self, message: &str, admin: bool) {
		let channel_id =
			if admin { self.admin_channel }
			else     { self.public_channel }
			.pipe(Id::new);

		_ = self.http
			.create_message(channel_id)
			.content(message)
			.await
			.inspect_err(|err| log_error("discord-create-message", err));
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