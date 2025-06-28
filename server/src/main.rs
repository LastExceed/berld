#![expect(incomplete_features, reason = "generic_const_exprs is incomplete, but works for our purposes")]
#![feature(generic_const_exprs)]
#![feature(future_join)]
#![feature(let_chains)]
#![feature(iter_collect_into)]
#![feature(iter_intersperse)]

#![allow(unreachable_pub, reason = "this isn't a lib, so adding `(crate)` to every `pub` is just pointless noise")]
#![allow(clippy::partial_pub_fields, reason = "OOP...")]

use colour::magenta_ln;
use config::{Config, ConfigError, File, Environment};
use server::Server;

mod server;
mod addon;

#[tokio::main]
async fn main() -> Result<(), ConfigError> {
	magenta_ln!("===== Berld =====");

	let config = Config::builder()
		.add_source(File::with_name("config"))
		.add_source(Environment::with_prefix("BERLD"))
		.build()?;

	Server::new(&config)?.run().await;
}