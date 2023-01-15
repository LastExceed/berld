#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(async_closure)]
#![feature(future_join)]
#![feature(let_chains)]

use colour::magenta_ln;

use server::Server;

mod server;
mod creature_id_pool;
mod player;
mod traffic_filter;
mod pvp;
mod creature;
mod handle_packet;
mod anti_cheat;

#[tokio::main]
async fn main() {
	magenta_ln!("===== Berld =====");
	Server::new().run().await;
}