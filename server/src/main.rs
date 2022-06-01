#![feature(generic_const_exprs)]
#![feature(option_result_contains)]

use server::Server;

mod server;
mod creature_id_pool;
mod player;

fn main() {
	println!("go");
	Server::new().run();
}

//todo
//nothing type?
//half open connection vulnerability
//lock scoping necessary?