use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;
use std::sync::Arc;
use std::time::Duration;

use config::{Config, ConfigError};
use rand::{random_range, rng};
use rand::seq::IndexedRandom;
use serde::Deserialize;
use tap::Tap;
use tokio::sync::RwLock;
use tokio::time::sleep;

use protocol::nalgebra::Point3;
use protocol::packet::{ChatMessageFromServer, CreatureUpdate, WorldUpdate};
use protocol::packet::common::{CreatureId, EulerAngles, Hitbox, Item, Race, item::{Kind, Material, kind}};
use protocol::packet::creature_update::{Affiliation, Occupation};
use protocol::packet::world_update::{Mission, Pickup, sound};
use protocol::packet::world_update::mission::{Objective, State};
use protocol::utils::constants::{SIZE_BLOCK, SIZE_ZONE, SIZE_SECTOR, VANILLA_PETS};
use protocol::utils::constants::materials::by_item_kind;
use protocol::utils::constants::rarity::{NORMAL, UNCOMMON, RARE, EPIC, LEGENDARY};
use protocol::utils::max_valid_item_level;

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
    kill_xp: i32,
    loot: LootWeights,
    rarity: RarityWeights
}

impl LegacyKoth {
    pub fn new(config: &Config) -> Result<Self, ConfigError> {
        let center: Option<Point3<i64>> = config_optional(config, "legacykoth.center")?
            .map(|raw: Point3<i64>| Point3::new(raw.x, raw.y, raw.z + LKOTH_HEIGHT_OFFSET));

        let radius_blocks: i64 = config_fallback(config, "legacykoth.radius", 30_i64)?;
        let interval_seconds: u64 = config_fallback(config, "legacykoth.interval", 5_u64)?;
        let reward_frequency: i32 = config_fallback(config, "legacykoth.reward_frequency", 420_i32)?;
        let king_reward_frequency: i32 = config_fallback(config, "legacykoth.king_reward_frequency", 180_i32)?;

        let loot: LootWeights = config_fallback(config, "legacykoth.loot", LootWeights::default())?;
        let rarity: RarityWeights = config_fallback(config, "legacykoth.rarity", RarityWeights::default())?;
        if loot.table().iter().all(|&(_, weight)| weight == 0) { return Err(ConfigError::Message("legacykoth.loot needs a non-zero weight".into())) }
        if rarity.table().iter().all(|&(_, weight)| weight == 0) { return Err(ConfigError::Message("legacykoth.rarity needs a non-zero weight".into())) }

        Ok(Self {
            points: RwLock::new(HashMap::new()),
            center,
            radius: radius_blocks * SIZE_BLOCK,
            interval: Duration::from_secs(interval_seconds),
            points_per_interval: REWARD_POINTS / (reward_frequency / interval_seconds as i32).max(1),
            king_points_per_interval: REWARD_POINTS / (king_reward_frequency / interval_seconds as i32).max(1),
            xp_per_interval: config_fallback(config, "legacykoth.xp_per_interval", 2_i32)?,
            king_xp_bonus: config_fallback(config, "legacykoth.king_xp_bonus", 5_i32)?,
            kill_king_points: config_fallback(config, "legacykoth.kill_king_points", 500_i32)?,
            kill_king_xp: config_fallback(config, "legacykoth.kill_king_xp", 20_i32)?,
            kill_points: config_fallback(config, "legacykoth.kill_points", 200_i32)?,
            kill_xp: config_fallback(config, "legacykoth.kill_xp", 10_i32)?,
            loot,
            rarity
        })
    }
}

pub fn start() {
    if SERVER.addons.events.legacy_koth.center.is_none() { return }

    tokio::spawn(async move {
        loop {
            sleep(SERVER.addons.events.legacy_koth.interval).await;
            lkoth_interval().await;
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
            new_king = Some((Arc::clone(player), new_total));
        }
    }

    let king_name = match &new_king {
        Some((player, _)) => player.character.read().await.name.chars().take(NAME_OVERFLOW).collect(),
        None => "KOTH".to_owned()
    };
    send_pillar_name(center, king_name).await;

    for (player, level) in &scorers {
        if *level < 500 {
            let is_king = Some(player.id) == king_id;
            give_xp(player, lkoth.xp_per_interval + if is_king { lkoth.king_xp_bonus } else { 0 }).await;
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
            scorers.push((Arc::clone(player), character.level));
        }
    }

    (scorers, online_ids)
}

fn find_king<'player>(candidates: impl Iterator<Item = &'player Arc<Player>>, points: &HashMap<CreatureId, i32>) -> Option<&'player Arc<Player>> {
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
            everyone.send_ignoring(&message).await;
        }
        give_reward(player).await;
    }
}

async fn give_reward(player: &Player) {
    let (level, occupation) = {
        let character = player.character.read().await;
        (character.level as i16, character.occupation)
    };
    let pickup = Pickup { interactor: player.id, item: reward_item(&SERVER.addons.events.legacy_koth, level, occupation) };
    play_sound_at_player(player, sound::Kind::Missioncomplete, 0.62, 1.0).await;
    player.send_ignoring(&WorldUpdate::from(pickup)).await;
}

#[derive(Debug, Clone, Copy)]
enum Loot { Weapon, Armor, Amulet, Ring, Leftovers, Spirit, Lamp, Pet }

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct LootWeights { weapon: u32, armor: u32, amulet: u32, ring: u32, leftovers: u32, spirit: u32, lamp: u32, pet: u32 }

impl Default for LootWeights {
    fn default() -> Self {
        Self { weapon: 25, armor: 25, amulet: 7, ring: 7, leftovers: 4, spirit: 25, lamp: 2, pet: 5 }
    }
}

impl LootWeights {
    const fn table(&self) -> [(Loot, u32); 8] {
        [(Loot::Weapon, self.weapon), (Loot::Armor, self.armor), (Loot::Amulet, self.amulet), (Loot::Ring, self.ring),
         (Loot::Leftovers, self.leftovers), (Loot::Spirit, self.spirit), (Loot::Lamp, self.lamp), (Loot::Pet, self.pet)]
    }
}

#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RarityWeights { normal: u32, uncommon: u32, rare: u32, epic: u32, legendary: u32 }

impl Default for RarityWeights {
    fn default() -> Self {
        Self { normal: 0, uncommon: 0, rare: 0, epic: 1, legendary: 1 }
    }
}

impl RarityWeights {
    const fn table(&self) -> [(u8, u32); 5] {
        [(NORMAL, self.normal), (UNCOMMON, self.uncommon), (RARE, self.rare), (EPIC, self.epic), (LEGENDARY, self.legendary)]
    }
}

fn pick_weighted<T: Copy>(table: &[(T, u32)]) -> Option<T> {
    table.choose_weighted(&mut rng(), |(_, weight)| *weight).ok().map(|(value, _)| *value)
}

type ClassWeapons = &'static [(kind::Weapon, &'static [Material])];

fn class_gear(occupation: Occupation) -> Option<(Material, ClassWeapons)> {
    use kind::Weapon::{Axe, Boomerang, Bow, Bracelet, Crossbow, Dagger, Fist, Greataxe, Greatmace, Greatsword, Longsword, Mace, Shield, Staff, Sword, Wand};
    use Material::{Cotton, Gold, Iron, Linen, Silk, Silver, Wood};

    match occupation {
        Occupation::Warrior => Some((Iron, &[(Sword, &[Iron]), (Axe, &[Iron]), (Mace, &[Iron]), (Shield, &[Iron]),
                                             (Greatsword, &[Iron]), (Greataxe, &[Iron]), (Greatmace, &[Iron, Wood])])),
        Occupation::Ranger  => Some((Linen, &[(Bow, &[Wood]), (Crossbow, &[Wood]), (Boomerang, &[Wood])])),
        Occupation::Mage    => Some((Silk, &[(Wand, &[Wood]), (Staff, &[Wood]), (Bracelet, &[Gold, Silver])])),
        Occupation::Rogue   => Some((Cotton, &[(Longsword, &[Iron]), (Dagger, &[Iron]), (Fist, &[Iron])])),
        _ => None
    }
}

fn reward_item(lkoth: &LegacyKoth, level: i16, occupation: Occupation) -> Item {
    let mut item = Item::default();

    let gear = class_gear(occupation);
    let table = lkoth.loot.table().map(|(loot, weight)| match loot {
        Loot::Weapon | Loot::Armor if gear.is_none() => (loot, 0),
        _ => (loot, weight)
    });

    (item.kind, item.material) = match (pick_weighted(&table), gear) {
        (Some(Loot::Weapon), Some((_, weapons))) => {
            let (weapon, materials) = pick_from(weapons);
            (Kind::Weapon(weapon), pick_from(materials))
        }
        (Some(Loot::Armor), Some((armor, _)))
            => (pick_from(&[Kind::Chest, Kind::Gloves, Kind::Boots, Kind::Shoulder]), armor),
        (loot, _) => {
            let kind = match loot {
                Some(Loot::Amulet)    => Kind::Amulet,
                Some(Loot::Ring)      => Kind::Ring,
                Some(Loot::Leftovers) => Kind::Leftovers,
                Some(Loot::Lamp)      => Kind::Lamp,
                Some(Loot::Pet)       => Kind::Pet(pick_from(&VANILLA_PETS)),
                _                     => Kind::Resource(kind::Resource::Spirit)
            };
            (kind, pick_from(by_item_kind(kind)))
        }
    };

    item.rarity = match item.kind {
        Kind::Weapon(_) | Kind::Chest | Kind::Gloves | Kind::Boots | Kind::Shoulder | Kind::Amulet | Kind::Ring | Kind::Leftovers
            => pick_weighted(&lkoth.rarity.table()).expect("validated at startup"),
        Kind::Resource(kind::Resource::Spirit) => RARE,
        _   => if item.kind.uses_rarity() { LEGENDARY } else { NORMAL }
    };

    item.level = max_valid_item_level(item.kind, level);
    item.seed = if item.uses_seed() { random_range(0..=i32::MAX) } else { 0 };

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

    killer.notify(message).await;
}

async fn send_pillar_name(center: Point3<i64>, name: String) {
    let update = CreatureUpdate {
        id: CreatureId(PILLAR_ID),
        name: Some(name),
        ..Default::default()
    };
    for player in SERVER.players.read().await.iter() {
        if is_in_zone(player.character.read().await.position, center, RENDER_DISTANCE_CREATURE) {
            player.send_ignoring(&update).await;
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
        player.send_ignoring(&packet).await;
    }

    let id = player.id;
    tokio::spawn(async move {
        sleep(Duration::from_secs(3)).await;
        if let Some(player) = SERVER.find_player_by_id(id).await {
            player.send_ignoring(&WorldUpdate::from(mission(center))).await;
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
                appearance: Some(appearance_invisible().tap_mut(|appearance| {
                    appearance.body_model = 2475;
                    appearance.creature_size = Hitbox { width: 1.0, depth: 1.0, height: 1.5 }
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
        appearance: Some(appearance_invisible().tap_mut(|appearance| {
            appearance.body_model = 2565;
            appearance.body_offset.z = 25.0;
            appearance.creature_size = Hitbox { width: 3.0, depth: 3.0, height: 4.0 }
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