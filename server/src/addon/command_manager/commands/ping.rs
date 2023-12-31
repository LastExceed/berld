use std::str::SplitWhitespace;
use std::sync::Arc;
use std::time::Duration;

use crate::addon::command_manager::{Command, CommandResult};
use crate::addon::command_manager::commands::Ping;
use crate::server::player::Player;
use crate::server::Server;
use ping_rs::*;

impl Command for Ping {
    const LITERAL: &'static str = "ping";
    const ADMIN_ONLY: bool = false;

    #[expect(clippy::significant_drop_in_scrutinee, clippy::significant_drop_tightening, reason = "cannot drop any earlier")]
    async fn execute<'fut>(&'fut self, server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
        let target = server
            .find_player(params.next().ok_or("no target specified")?).await
            .ok_or("target not found")?;
        let target_ip = target.address.ip();

        let ping_reply = send_ping_async(&target_ip, Duration::from_secs(5), Arc::new(&[]), None)
            .await.map_err(|_|"ping TimedOut or was ignored by the firewall")?;
        let display = format!("{} round trip latency: {}ms", target.character.read().await.name, ping_reply.rtt);

        Ok(Some(display))
    }
}