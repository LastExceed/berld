#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(async_closure)]
#![feature(future_join)]
#![feature(let_chains)]
#![feature(async_fn_in_trait)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(iter_collect_into)]

use colour::magenta_ln;

use server::Server;

mod server;
mod addon;

#[tokio::main]
async fn main() {
	magenta_ln!("===== Berld =====");
	Server::new().run().await;
}