use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use crate::server::player::Player;
use crate::server::Server;
use crate::server::utils::give_xp;

type CommandFuture<'a> = Pin<Box<dyn Future<Output=CommandResult> + Send + 'a>>;
type CommandResult = Result<(), &'static str>;

pub struct CommandManager {
	commands: HashMap<&'static str, Box<dyn ObjectSafeCommand>>
}

impl CommandManager {
	pub fn new() -> Self {
		Self {
			commands: HashMap::from([
				("xp", Box::new(Xp) as Box<dyn ObjectSafeCommand>)
			])
		}
	}

	pub async fn on_chat_message(&self, server: &Server, caller: &Player, text: &str) -> bool {
		if !text.starts_with('/') {
			return false;
		}

		let result = text[1..]
			.split_whitespace()
			.next()
			.ok_or("no command specified")
			.and_then(|frag| self.commands.get(frag).ok_or("unknown command"));
//			.and_then(|command| command.get_execution_future(server, caller).await)
//			.map_err(async move |error| caller.notify(error).await)

		let result = match result {//unfortunately mapping functions dont support async/await, so we need to fallback to `match`
			Ok(command) => command.get_execution_future(server, caller).await,
			Err(e) => Err(e)
		};

		if let Err(error) = result {
			caller.notify(error).await;
		}

		true
	}
}

pub trait Command {
	const ADMIN_ONLY: bool;

	fn execute<'fut>(&'fut self, server: &'fut Server, caller: &'fut Player) -> impl Future<Output=CommandResult> + Send + 'fut;//if you see an error here, ignore it -> https://github.com/intellij-rust/intellij-rust/issues/10216
}

//neither associated constants nor async functions are object safe, so we need a proxy for both
trait ObjectSafeCommand: Send + Sync {//todo: Sync bound is only because of discord spaghetti {
	fn get_admin_only(&self) -> bool;

	fn get_execution_future<'fut>(&'fut self, server: &'fut Server, caller: &'fut Player) -> CommandFuture<'fut>;
}

impl<T: Command + Send + Sync> ObjectSafeCommand for T {
	fn get_admin_only(&self) -> bool {
		T::ADMIN_ONLY
	}

	fn get_execution_future<'fut>(&'fut self, server: &'fut Server, caller: &'fut Player) -> CommandFuture<'fut> {
		Box::pin(self.execute(server, caller))
	}
}


struct Xp;

impl Command for Xp {
	const ADMIN_ONLY: bool = false;

	async fn execute(&self, _server: &Server, caller: &Player) -> CommandResult {
		give_xp(caller, 42).await;

		Ok(())
	}
}