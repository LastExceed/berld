#![expect(incomplete_features, reason = "generic_const_exprs is incomplete, but works for our purposes")]
#![feature(generic_const_exprs)]
#![feature(future_join)]
#![feature(iter_collect_into)]
#![feature(iter_intersperse)]

#![allow(unreachable_pub, reason = "this isn't a lib, so adding `(crate)` to every `pub` is just pointless noise")]
#![allow(clippy::partial_pub_fields, reason = "OOP...")]

use std::sync::LazyLock;

use colour::magenta_ln;
use config::{Config, File, Environment};
use server::Server;
use tap::Pipe;

mod server;
mod addon;

static SERVER: LazyLock<Server> = LazyLock::new(||
	Config::builder()
		.add_source(File::with_name("config"))
		.add_source(Environment::with_prefix("BERLD"))
		.build()
		.unwrap()
		.pipe_ref(Server::new)
		.unwrap()
);

#[tokio::main]
async fn main() {
	magenta_ln!("===== Berld =====");

	SERVER.run().await;
}