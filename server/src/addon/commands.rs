use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::str::SplitWhitespace;

use protocol::packet::{CreatureUpdate, WorldUpdate};
use protocol::packet::common::CreatureId;
use protocol::packet::creature_update::Affiliation;
use protocol::packet::world_update::Kill;

use crate::server::player::Player;

type ParseFn = for <'a> fn(&'a Player, &mut SplitWhitespace) -> ParseResult<'a>;
type ParseResult<'a> = Result<CmdFut<'a>, &'static str>;
type CmdFut<'a> = Pin<Box<dyn Future<Output=()> + Send + 'a>>;

pub struct CommandManager {
	pub commands: HashMap<&'static str, ParseFn>
}

impl CommandManager {
	pub fn new() -> Self {
		let mut commands: HashMap<&'static str, ParseFn> = HashMap::new();
		commands.insert("xp", xp_command);
		Self {
			commands
		}
	}

	pub async fn on_chat_message(&self, source: &Player, text: &str) -> bool {
		if !text.starts_with('/') {
			return false;
		}

		let mut fragments = text[1..].split_whitespace();

		let Some(command) = fragments.next()
			else {
				//todo: empty command
				return true;
			};

		let Some(parse_command) = self.commands.get(command)
			else {
				source.notify("unknown command").await;
				return true;
			};

		let parse_result = parse_command(source, &mut fragments);

		if fragments.next().is_some() {
			source.notify("too many arguments").await;
			return true;
		}

		match parse_result {
			Ok(command_future) => command_future.await,
			Err(message) => source.notify(message).await,
		}

		true
	}
}

fn xp_command<'a>(source: &'a Player, params: &mut SplitWhitespace) -> Result<CmdFut<'a>, &'static str> {
	let amount = params
		.next().ok_or("too few arguments")?
		.parse().map_err(|_| "invalid amount")?;

	Ok(Box::pin(give_xp(source, amount)))
}

async fn give_xp(source: &Player, experience: i32) {//todo: move to server utils or sth
	let dummy = CreatureUpdate {
		id: CreatureId(9999),
		affiliation: Some(Affiliation::Enemy),
		..Default::default()
	};
	source.send_ignoring(&dummy).await;

	let kill = Kill {
		killer: source.id,
		unknown: 0,
		victim: dummy.id,
		experience
	};

	source.send_ignoring(&WorldUpdate::from(kill)).await;
}