use std::sync::Arc;
use std::time::{Duration, Instant};

use protocol::packet::CreatureUpdate;

use crate::addon::events;
use crate::server::{player::Player, Server};

pub async fn on_creature_update(server: &Server, source: &Player, packet: &CreatureUpdate) {
    if !packet.health.is_some_and(|value| value <= 0.0) {
        return;
    }

    let victim = source.character.read().await.name.clone();

    let last_attacker = source
        .addon_data
        .write()
        .await
        .last_attacker
        .take()
        .filter(|(timestamp, _name)| timestamp.elapsed() < Duration::from_secs(1));

    let message = match &last_attacker {
        Some((_, name)) => format!("{name} killed {victim}"),
        None => format!("{victim} died")
    };
    server.announce(message).await;

    if let Some((_, attacker_name)) = last_attacker
        && let Some(killer) = find_player_by_name(server, &attacker_name).await
    {
        events::on_kill(server, &killer, source).await;
    }
}

pub async fn set_last_attacker(target: &Player, attacker_name: String) {
    target
        .addon_data
        .write()
        .await
        .last_attacker = Some((Instant::now(), attacker_name));
}

async fn find_player_by_name(server: &Server, name: &str) -> Option<Arc<Player>> {
    for player in server.players.read().await.iter() {
        if player.character.read().await.name == name {
            return Some(Arc::clone(player));
        }
    }
    None
}