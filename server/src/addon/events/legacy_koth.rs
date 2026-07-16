use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::sync::Arc;
use std::time::Duration;

use config::{Config, ConfigError};
use rand::random;
use strum::IntoEnumIterator;
use tap::Tap;
use tokio::sync::RwLock;
use tokio::time::sleep;

use protocol::nalgebra::Point3;
use protocol::packet::{ChatMessageFromServer, CreatureUpdate, WorldUpdate};
use protocol::packet::common::{CreatureId, EulerAngles, Hitbox, Item, Race, item::{Kind, kind}};
use protocol::packet::creature_update::Affiliation;
use protocol::packet::world_update::{Mission, Pickup, sound};
use protocol::packet::world_update::mission::{Objective, State};
use protocol::utils::constants::{SIZE_BLOCK, SIZE_ZONE, SIZE_SECTOR};
use protocol::utils::constants::materials::by_item_kind;
use protocol::utils::constants::rarity::{NORMAL, RARE, EPIC, LEGENDARY};

use crate::addon::events::utils::{appearance_invisible, config_fallback, config_optional, creatures_circular, is_in_zone, pick_from, NAME_OVERFLOW, RENDER_DISTANCE_CREATURE};
use crate::addon::play_sound_at_player;
use crate::server::{Server, player::Player, utils::give_xp};
use crate::SERVER;

const LKOTH_HEIGHT_OFFSET: i64 = 100000;
const LKOTH_TORCH_SPACING: i64 = SIZE_BLOCK * 20;
const PILLAR_ID: i64 = 50000;
const TORCHES_ID: i64 = 75000;

const REWARD_POINTS: i32 = 10000;
const REWARD_THRESHOLDS: [i32; 4] = [20, 40, 60, 80];

const REWARD_WEAPON_ODDS: f32 = 0.30;
const REWARD_ARMOR_ODDS: f32 = 0.30;
const REWARD_SPIRIT_ODDS: f32 = 0.35;

#[derive(Debug)]
pub struct LegacyKoth {
    points: RwLock<HashMap<CreatureId, i32>>,
    center: Option<Point3<i64>>,
    radius: i64,
    interval: Duration,
    points_per_interval: i32,
    king_points_per_interval: i32,
    xp_per_interval: i32,
    king_xp_bonus: i32,
    kill_king_points: i32,
    kill_king_xp: i32,
    kill_points: i32,
    kill_xp: i32
}

impl LegacyKoth {
    pub fn new(config: &Config) -> Result<Self, ConfigError> {
        let center: Option<Point3<i64>> = config_optional(config, "legacykoth.center")?
            .map(|raw: Point3<i64>| Point3::new(raw.x, raw.y, raw.z + LKOTH_HEIGHT_OFFSET));

        let radius_blocks: i64 = config_fallback(config, "legacykoth.radius", 30i64)?;
        let interval_seconds: u64 = config_fallback(config, "legacykoth.interval", 5u64)?;
        let reward_frequency: i32 = config_fallback(config, "legacykoth.reward_frequency", 420i32)?;
        let king_reward_frequency: i32 = config_fallback(config, "legacykoth.king_reward_frequency", 180i32)?;

        Ok(Self {
            points: RwLock::new(HashMap::new()),
            center,
            radius: radius_blocks * SIZE_BLOCK,
            interval: Duration::from_secs(interval_seconds),
            points_per_interval: REWARD_POINTS / (reward_frequency / interval_seconds as i32).max(1),
            king_points_per_interval: REWARD_POINTS / (king_reward_frequency / interval_seconds as i32).max(1),
            xp_per_interval: config_fallback(config, "legacykoth.xp_per_interval", 2i32)?,
            king_xp_bonus: config_fallback(config, "legacykoth.king_xp_bonus", 5i32)?,
            kill_king_points: config_fallback(config, "legacykoth.kill_king_points", 500i32)?,
            kill_king_xp: config_fallback(config, "legacykoth.kill_king_xp", 20i32)?,
            kill_points: config_fallback(config, "legacykoth.kill_points", 200i32)?,
            kill_xp: config_fallback(config, "legacykoth.kill_xp", 10i32)?
        })
    }
}

pub fn start() {
    if SERVER.addons.events.legacy_koth.center.is_none() { return }

    tokio::spawn(async move {
        loop {
            sleep(SERVER.addons.events.legacy_koth.interval).await;
            lkoth_interval().await
        }
    });
}

async fn lkoth_interval() {
    let lkoth = &SERVER.addons.events.legacy_koth;
    let center = lkoth.center.expect("only runs when legacy koth is enabled");

    let (scorers, online_ids) = scan_players(&SERVER, center, lkoth.radius).await;
    lkoth.points.write().await.retain(|id, _| online_ids.contains(id));

    let king_id = {
        let points = lkoth.points.read().await;
        find_king(scorers.iter().map(|(player, _)| player), &points).map(|player| player.id)
    };

    let mut new_king: Option<(Arc<Player>, i32)> = None;
    for (player, _) in &scorers {
        let amount = if Some(player.id) == king_id { lkoth.king_points_per_interval } else { lkoth.points_per_interval };
        let (new_total, threshold, reward) = add_points(&mut *lkoth.points.write().await, player.id, amount);
        handle_points(player, threshold, reward).await;

        if new_king.as_ref().is_none_or(|(_, best)| new_total >= *best) {
            new_king = Some((Arc::clone(player), new_total))
        }
    }

    let king_name = match &new_king {
        Some((player, _)) => player.character.read().await.name.chars().take(NAME_OVERFLOW).collect(),
        None => "KOTH".to_string()
    };
    send_pillar_name(center, king_name).await;

    for (player, level) in &scorers {
        if *level < 500 {
            let is_king = Some(player.id) == king_id;
            give_xp(player, lkoth.xp_per_interval + if is_king { lkoth.king_xp_bonus } else { 0 }).await
        }
    }
}

async fn scan_players(server: &Server, center: Point3<i64>, radius: i64) -> (Vec<(Arc<Player>, i32)>, HashSet<CreatureId>) {
    let players = server.players.read().await;
    let online_ids = players.iter().map(|player| player.id).collect();

    let mut scorers = Vec::new();
    for player in players.iter() {
        let character = player.character.read().await;
        if character.health > 0.0 && is_in_zone(character.position, center, radius) {
            scorers.push((Arc::clone(player), character.level))
        }
    }

    (scorers, online_ids)
}

fn find_king<'p>(candidates: impl Iterator<Item = &'p Arc<Player>>, points: &HashMap<CreatureId, i32>) -> Option<&'p Arc<Player>> {
    candidates.max_by_key(|player| points.get(&player.id).copied().unwrap_or(0))
}

async fn current_king(server: &Server) -> Option<Arc<Player>> {
    let lkoth = &server.addons.events.legacy_koth;
    let center = lkoth.center?;

    let (scorers, _) = scan_players(server, center, lkoth.radius).await;
    let points = lkoth.points.read().await;
    find_king(scorers.iter().map(|(player, _)| player), &points).cloned()
}

fn add_points(points: &mut HashMap<CreatureId, i32>, id: CreatureId, amount: i32) -> (i32, Option<i32>, bool) {
    let old_points = points.get(&id).copied().unwrap_or(0);
    let mut new_points = old_points + amount;

    let threshold = REWARD_THRESHOLDS
        .into_iter()
        .rfind(|&threshold_percentage| {
            let bound = REWARD_POINTS * threshold_percentage / 100;
            old_points < bound && new_points >= bound
        })
        .map(|_| new_points);

    let reward = new_points >= REWARD_POINTS;
    if reward { new_points -= REWARD_POINTS }

    points.insert(id, new_points);
    (new_points, threshold, reward)
}

async fn handle_points(player: &Player, threshold: Option<i32>, reward: bool) {
    if let Some(points) = threshold {
        let percent = points as f64 / REWARD_POINTS as f64 * 100.0;
        player.notify(format!("KotH points {points}/{REWARD_POINTS} ({percent:.1}%)")).await;
    }
    if reward {
        let name = player.character.read().await.name.clone();
        let message = ChatMessageFromServer {
            source: CreatureId(0),
            text: format!("{name} has reached {REWARD_POINTS} points, and receives an additional reward!")
        };
        for everyone in SERVER.players.read().await.iter() {
            everyone.send_ignoring(&message).await
        }
        give_reward(player).await
    }
}

async fn give_reward(player: &Player) {
    let level = player.character.read().await.level as i16;
    let pickup = Pickup { interactor: player.id, item: reward_item(level) };
    play_sound_at_player(player, sound::Kind::Missioncomplete, 0.62, 1.0).await;
    player.send_ignoring(&WorldUpdate::from(pickup)).await
}

fn reward_item(level: i16) -> Item {
    let mut item = Item::default();

    item.kind = match random::<f32>() {
        roll if roll < REWARD_WEAPON_ODDS
            => Kind::Weapon(pick_from(&kind::Weapon::iter().collect::<Vec<_>>())),
        roll if roll < REWARD_WEAPON_ODDS + REWARD_ARMOR_ODDS
            => pick_from(&[Kind::Chest, Kind::Gloves, Kind::Boots, Kind::Shoulder]),
        roll if roll < REWARD_WEAPON_ODDS + REWARD_ARMOR_ODDS + REWARD_SPIRIT_ODDS
            => Kind::Resource(kind::Resource::Spirit),
        _   => Kind::Pet(pick_from(&Race::iter().collect::<Vec<_>>()))
    };

    item.rarity = match item.kind {
        Kind::Weapon(_) | Kind::Chest | Kind::Gloves | Kind::Boots | Kind::Shoulder
            => pick_from(&[EPIC, LEGENDARY]),
        Kind::Resource(kind::Resource::Spirit) => RARE,
        _   => if item.kind.uses_rarity() { LEGENDARY } else { NORMAL }
    };

    item.material = pick_from(by_item_kind(item.kind));
    item.level = if item.kind.uses_level() { level } else { 1 };
    item.seed = if item.kind.uses_seed() { random() } else { 0 };

    item
}

pub async fn on_kill(server: &Server, killer: &Player, victim: &Player) {
    let lkoth = &server.addons.events.legacy_koth;
    let Some(center) = lkoth.center else { return };

    let killer_character = killer.character.read().await;
    if !is_in_zone(killer_character.position, center, lkoth.radius) { return }
    let killer_level = killer_character.level;
    drop(killer_character);

    let is_king_kill = current_king(server).await.is_some_and(|king| king.id == victim.id);
    let (points, xp, message) = if is_king_kill {
        let points = lkoth.kill_king_points;
        let xp = lkoth.kill_king_xp;
        (points, xp, format!("you gain {points}(+{xp}xp) KotH points! (+king bonus)"))
    } else {
        let points = lkoth.kill_points;
        let xp = lkoth.kill_xp;
        (points, xp, format!("you gain {points}(+{xp}xp) KotH points!"))
    };

    let (_, threshold, reward) = add_points(&mut *lkoth.points.write().await, killer.id, points);
    handle_points(killer, threshold, reward).await;

    if killer_level < 500 { give_xp(killer, xp).await }

    killer.notify(message).await
}

async fn send_pillar_name(center: Point3<i64>, name: String) {
    let update = CreatureUpdate {
        id: CreatureId(PILLAR_ID),
        name: Some(name),
        ..Default::default()
    };
    for player in SERVER.players.read().await.iter() {
        if is_in_zone(player.character.read().await.position, center, RENDER_DISTANCE_CREATURE) {
            player.send_ignoring(&update).await
        }
    }
}

pub async fn on_join(player: &Player) {
    let Some(center) = SERVER.addons.events.legacy_koth.center else { return };
    let radius = SERVER.addons.events.legacy_koth.radius;

    let amount = torches_amount(radius);
    let mut creatures = Vec::with_capacity(1 + amount);
    creatures.push(pillar(center));
    creatures.extend(torches(center, radius, amount));

    for packet in creatures {
        player.send_ignoring(&packet).await
    }

    let id = player.id;
    tokio::spawn(async move {
        sleep(Duration::from_secs(3)).await;
        if let Some(player) = SERVER.find_player_by_id(id).await {
            player.send_ignoring(&WorldUpdate::from(mission(center))).await
        }
    });
}

fn mission(center: Point3<i64>) -> Mission {
    let sizing = |size: i64| center.xy().map(|coord| (coord / size) as i32);

    Mission {
        sector: sizing(SIZE_SECTOR),
        unknown_a: 1,
        unknown_b: 1,
        unknown_c: 1,
        id: i32::MAX,
        objective: Objective::Monster,
        race: Race::Bandit,
        level: 500,
        rarity: LEGENDARY,
        state: State::InProgress,
        progress_current: 100,
        progress_maximum: 100,
        zone: sizing(SIZE_ZONE)
    }
}

fn torches_amount(radius: i64) -> usize {
    let circumference = 2.0 * PI * (radius as f64);
    let count = (circumference / (LKOTH_TORCH_SPACING as f64)).floor() as usize;
    if count < 10 { 10 } else { count }
}

fn torches(center: Point3<i64>, radius: i64, count: usize) -> Vec<CreatureUpdate> {
    (0..count)
        .map(|i| {
            let (position, yaw) = creatures_circular(center, radius, count, i);

            CreatureUpdate {
                appearance: Some(appearance_invisible().tap_mut(|a| {
                    a.body_model = 2475;
                    a.creature_size = Hitbox { width: 1.0, depth: 1.0, height: 1.5 }
                })),
                id: CreatureId(TORCHES_ID + i as i64),
                race: Some(Race::DepositSapphire),
                name: Some("King ofthe Hill".into()),
                level: Some(i32::MAX),
                health: Some(f32::MAX),
                master: Some(CreatureId(i64::MAX)),
                affiliation: Some(Affiliation::Pet),
                rotation: Some(EulerAngles { pitch: 0.0, roll: 0.0, yaw }),
                position: Some(position),
                ..Default::default()
            }
        })
        .collect()
}

fn pillar(center: Point3<i64>) -> CreatureUpdate {
    CreatureUpdate {
        appearance: Some(appearance_invisible().tap_mut(|a| {
            a.body_model = 2565;
            a.body_offset.z = 25.0;
            a.creature_size = Hitbox { width: 3.0, depth: 3.0, height: 4.0 }
        })),
        id: CreatureId(PILLAR_ID),
        race: Some(Race::DepositDiamond),
        name: Some("KOTH".into()),
        level: Some(i32::MAX),
        health: Some(f32::MAX),
        master: Some(CreatureId(i64::MAX)),
        affiliation: Some(Affiliation::Pet),
        position: Some(center),
        ..Default::default()
    }
}