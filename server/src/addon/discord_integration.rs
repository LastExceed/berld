use std::time::Duration;

use config::{Config, ConfigError};
use itertools::Itertools as _;
use tap::Pipe as _;
use tokio::sync::mpsc;
use tokio::time::sleep;
use twilight_gateway::{EventTypeFlags, Shard, StreamExt as _};
use twilight_http::Client;
use twilight_model::gateway::{Intents, ShardId};
use twilight_model::gateway::event::Event::MessageCreate;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;

use protocol::packet::ChatMessageFromServer;
use protocol::packet::common::CreatureId;
use crate::SERVER;
use crate::{addon::command_manager::CommandResult, server::utils::log_error};
use crate::server::utils::extend_lifetime;

use crate::server::Server;

#[derive(Debug)]
pub struct DiscordIntegration {
	token: String,
	public_channel: Id<ChannelMarker>,
	admin_channel: Id<ChannelMarker>,
	queue: mpsc::Sender<(String, bool)>
}

impl DiscordIntegration {
	pub fn new(config: &Config) -> Result<Self, ConfigError> {
		let token = config.get_string("discord_bot_token")?;

		let (tx, mut rx) = mpsc::channel(1000);
		let http = Client::new(token.clone());
		let public_channel = config.get::<u64>("discord_public_channel_id")?.pipe(Id::new);
		let  admin_channel = config.get::<u64>( "discord_admin_channel_id")?.pipe(Id::new);
		
		let instance = Self {
			token,
			public_channel,
			admin_channel,
			queue: tx
		};
		
		tokio::spawn(async move {
			#[expect(clippy::infinite_loop, reason = "fix is very verbose")]
			loop {
				let mut buffer = vec![];
				let _n = rx
					.recv_many(&mut buffer, 1000)
					.await;
				
				bulk_post(&http, &buffer, true ,  admin_channel).await;
				bulk_post(&http, &buffer, false, public_channel).await;
				
				sleep(Duration::from_secs(1)).await;
			}
		});

		Ok(instance)
	}
}

async fn bulk_post(http: &Client, messages: &[(String, bool)], admin: bool, channel: Id<ChannelMarker>) {
	let message = messages
    	.iter()
    	.filter(|(_msg, is_admin)| *is_admin == admin)
    	.map(|(msg, _)| msg)
    	.join("\n");
	
	_ = http.create_message(channel)
		.content(&message)
		.await
		.inspect_err(|err| log_error("discord-create-message", err));
}

impl DiscordIntegration {
	pub fn run(&self) {
		let mut shard = Shard::new(
			ShardId::ONE,
			self.token.clone(),
			Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT
		);

		let self_static = extend_lifetime(self);

		tokio::spawn(async move {
			#[expect(clippy::infinite_loop, reason = "way too verbose to fix")]
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

						let callback = |response| { Self::command_callback(&SERVER, response, admin) };

						let is_command = SERVER.addons.command_manager.on_message(
							&SERVER,
							None,
							admin,
							&message.content,
							'.',
							callback
						).await;

						if is_command {
							continue;
						}

						SERVER.broadcast(&ChatMessageFromServer {//dont use server.announce() as that would cause an echo
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
		self.queue
			.send((message.into(), admin))
			.await
			.unwrap();
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