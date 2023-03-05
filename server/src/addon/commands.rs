use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::str::SplitWhitespace;

use crate::server::player::Player;
use crate::server::utils::give_xp;

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