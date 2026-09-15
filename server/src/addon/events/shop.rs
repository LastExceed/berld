use std::collections::{HashMap, HashSet};
use std::mem;
use std::ops::Range;

use config::{Config, ConfigError};
use strum::IntoEnumIterator;
use tap::Tap;
use tokio::sync::RwLock;

use protocol::nalgebra::Point3;
use protocol::packet::common::*;
use protocol::packet::common::item::*;
use protocol::packet::creature_update::*;
use protocol::packet::creature_update::equipment::Slot;
use protocol::packet::world_update::{Pickup, sound};
use protocol::packet::WorldUpdate;
use protocol::packet::CreatureUpdate;
use protocol::utils::constants::{materials, SIZE_BLOCK, SIZE_ZONE};
use protocol::utils::constants::rarity::*;

use crate::addon::events::utils::{appearance_invisible, config_fallback, config_optional, creatures_circular, NAME_OVERFLOW};
use crate::addon::play_sound_at_player;
use crate::server::player::Player;
use crate::server::Server;
use crate::SERVER;

const SHOP_RADIUS_BLOCKS: i64 = 15;
const SHOP_HITBOX: Hitbox = Hitbox { width: 1.5, depth: 1.5, height: 2.5 };
const SHOP_INDEX: i32 = 1000;
const SHOP_ID: i64 = 100000;
const KEEPER_INDEX: i32 = 2000;
const KEEPER_ID: i64 = 200000;
const DISABLED_ITEMS: [Kind; 2] = [Kind::PlatinumCoin, Kind::ManaCube];

#[derive(Default, Clone, Copy)]
pub enum State {
    #[default]
    MainType,
    SubType,
    Material,
    Rarity,
    Model,
    Stat,
    Complete,
}

#[derive(Default)]
pub struct Session {
    pub state: State,
    pub item: Item,
    pub npcs: Vec<CreatureUpdate>
}

pub struct Shop {
    pub center: Option<Point3<i64>>,
    pub radius: i64,
    pub sessions: RwLock<HashMap<CreatureId, Session>>,
}

impl Shop {
    pub fn new(config: &Config) -> Result<Self, ConfigError> {
        Ok(Self {
            center: config_optional(config, "shop.center")?,
            radius: config_fallback(config, "shop.radius", SHOP_RADIUS_BLOCKS)? * SIZE_BLOCK,
            sessions: RwLock::new(HashMap::new())
        })
    }

    pub async fn shop_keeper(&self, player: &Player) {
        let Some(center) = self.center else { return };

        let shop_keeper = CreatureUpdate {
            appearance: Some(Appearance { body_model: 2111, creature_size: SHOP_HITBOX, ..appearance_invisible() }),
            id: CreatureId(KEEPER_ID),
            race: Some(Race::Bandit),
            name: Some("Item\nShop".into()),
            position: Some(center),
            zone_data_index: Some(Point3::new(
                (center.x / SIZE_ZONE) as i32,
                (center.y / SIZE_ZONE) as i32,
                KEEPER_INDEX)),
            ..Default::default()
        };

        player.send_ignoring(&shop_keeper).await;
    }

    pub async fn interaction(&self, server: &Server, player: &Player, index: Point3<i32>) {
        let Some(center) = self.center else { return };
        self.cleanup_stale_sessions(server).await;

        if index == Point3::new((center.x / SIZE_ZONE) as i32, (center.y / SIZE_ZONE) as i32, KEEPER_INDEX) {
            play_sound_at_player(player, sound::Kind::CraftProc, 1.0, 1.0).await;
            self.reset_session(player).await;
            self.update_shop(player, center).await;
            return
        }

        let is_shop_npc = self.sessions
            .read()
            .await
            .get(&player.id)
            .is_some_and(|session| session.npcs.iter().any(|cu| cu.zone_data_index == Some(index)));

        if !is_shop_npc {
            return
        }

        let option = index.z - SHOP_INDEX;
        let player_level = player.character.read().await.level as i16;

        let item_state: Result<Option<Box<Item>>, String> = {
            let mut sessions = self.sessions.write().await;
            let session = sessions.entry(player.id).or_default();
            item_selection(&mut session.state, &mut session.item, option);
            item_validation(&mut session.state, &mut session.item, player_level)
                .map(|()| matches!(session.state, State::Complete).then(|| Box::new(session.item.clone())))
        };

        match item_state {
            Err(reason)    => {play_sound_at_player(player, sound::Kind::SpikeTrap, 1.0, 1.0).await;
                               player.notify(reason).await;
                               self.reset_session(player).await;}
            Ok(Some(item)) => {play_sound_at_player(player, sound::Kind::DropCoin, 1.0, 1.0).await;
                               let pickup = Pickup { interactor: player.id, item: *item };
                               player.send_ignoring(&WorldUpdate::from(pickup)).await;
                               self.reset_session(player).await;}
            Ok(None)       =>  play_sound_at_player(player, sound::Kind::Craft, 1.0, 1.0).await
        }

        self.update_shop(player, center).await;
    }

    async fn reset_session(&self, player: &Player) {
        let mut sessions = self.sessions.write().await;
        let session = sessions.entry(player.id).or_default();
        session.state = State::default();
        session.item = Item::default();
    }

    async fn cleanup_stale_sessions(&self, server: &Server) {
        let online_ids: HashSet<CreatureId> = server.players
            .read()
            .await
            .iter()
            .map(|player| player.id)
            .collect();

        self.sessions.write().await.retain(|id, _| online_ids.contains(id));
    }

    async fn update_shop(&self, player: &Player, center: Point3<i64>) {
        let (old_npcs, new_npcs) = {
            let mut sessions = self.sessions.write().await;
            let session = sessions.entry(player.id).or_default();

            let old_npcs = mem::take(&mut session.npcs);
            let options = shop_options(session.state, &session.item);
            let new_npcs = shop_npcs(center, self.radius, &options, session.state, &session.item);
            session.npcs.clone_from(&new_npcs);

            (old_npcs, new_npcs)
        };

        for packet in &old_npcs {
            player.send_ignoring(&CreatureUpdate {
                id: packet.id,
                health: Some(0.0),
                ..Default::default()
            }).await;
        }

        for packet in &new_npcs {
            player.send_ignoring(packet).await;
        }
    }
}

pub async fn on_join(player: &Player) {
    SERVER.addons.events.shop.shop_keeper(player).await;
}

fn shop_npcs(center: Point3<i64>, radius: i64, options: &[i32], state: State, item: &Item) -> Vec<CreatureUpdate> {
    options
        .iter()
        .enumerate()
        .map(|(i, &option)| {
            let (position, yaw) = creatures_circular(center, radius, options.len(), i);

            let preview = item_preview(state, item, option).unwrap_or_else(|| item.clone());
            let mut equipment = Equipment::default();
            equipment[Slot::Chest] = preview;

            CreatureUpdate {
                appearance: Some(appearance_invisible().tap_mut(|appearance| {
                    appearance.body_model = 2316;
                    appearance.body_offset.z = 5.0;
                    appearance.creature_size = SHOP_HITBOX;
                })),
                id: CreatureId(SHOP_ID + i as i64),
                race: Some(Race::Bandit),
                name: Some(npc_names(state, item, option).chars().take(NAME_OVERFLOW).collect()),
                health: Some(0.0001),
                rotation: Some(EulerAngles { pitch: 0.0, roll: 0.0, yaw }),
                position: Some(position),
                zone_data_index: Some(Point3::new(
                    (position.x / SIZE_ZONE) as i32,
                    (position.y / SIZE_ZONE) as i32,
                    SHOP_INDEX + option)),
                equipment: Some(equipment),
                ..Default::default()
            }
        })
        .collect()
}

fn maintypes(option: i32) -> Option<Kind> {
    Kind::iter().find(|kind| KindDiscriminants::from(*kind) as u8 == option as u8)
}

fn subtypes(kind: Kind) -> Vec<(i32, Kind)> {
    match kind {
        Kind::Consumable(_) => kind::Consumable::iter().map(|consumable| (consumable as i32, Kind::Consumable(consumable))).collect(),
        Kind::Weapon(_)     => kind::Weapon::iter().map(|weapon| (weapon as i32, Kind::Weapon(weapon))).collect(),
        Kind::Resource(_)   => kind::Resource::iter().map(|resource| (resource as i32, Kind::Resource(resource))).collect(),
        Kind::Candle(_)     => kind::Candle::iter().map(|candle| (candle as i32, Kind::Candle(candle))).collect(),
        Kind::Pet(_)        => Race::iter().map(|race| (race as i32, Kind::Pet(race))).collect(),
        Kind::PetFood(_)    => Race::iter().map(|race| (race as i32, Kind::PetFood(race))).collect(),
        Kind::Quest(_)      => kind::Quest::iter().map(|quest| (quest as i32, Kind::Quest(quest))).collect(),
        Kind::Special(_)    => kind::Special::iter().map(|special| (special as i32, Kind::Special(special))).collect(),
        _                   => Vec::new()
    }
}

fn model_seeds(item: &Item) -> Range<i32> {
    let count = item.model_count();
    if matches!(item.kind, Kind::Weapon(_)) { -(count - 1)..count } else { 0..count }
}

fn stat_seed(item: &Item, option: i32) -> Option<i32> {
    let count = item.model_count();
    let step = if item.seed < 0 { -count } else { count };
    (0..SUB_STATS)
        .map(|i| item.seed + step * i)
        .find(|&seed| sub_stat(seed) == option)
}

fn shop_options(state: State, item: &Item) -> Vec<i32> {
    match state {
        State::MainType => Kind::iter().map(|kind| KindDiscriminants::from(kind) as i32).collect(),
        State::SubType  => subtypes(item.kind).into_iter().map(|(subtype, _)| subtype).collect(),
        State::Material => materials::by_item_kind(item.kind).iter().map(|&material| material as i32).collect(),
        State::Rarity   => [NORMAL, UNCOMMON, RARE, EPIC, LEGENDARY].iter().map(|&rarity| rarity as i32).collect(),
        State::Model    => model_seeds(item).collect(),
        State::Stat     => (0..SUB_STATS).filter(|&option| stat_seed(item, option).is_some()).collect(), // Bracelet cannot reach all 21 stat variants
        State::Complete => Vec::new()
    }
}

#[expect(clippy::match_same_arms, reason = "hack")]
fn item_selection(state: &mut State, item: &mut Item, option: i32) {
    if let Some(preview) = item_preview(*state, item, option) {
        *item = preview;
        *state = match *state {
            State::MainType => State::SubType,
            State::SubType  => State::Material,
            State::Material => State::Rarity,
            State::Rarity   => State::Model,
            State::Model    => State::Stat,
            State::Stat     => State::Complete,
            State::Complete => State::Complete
        }
    }
}

fn item_preview(state: State, item: &Item, option: i32) -> Option<Item> {
    let mut preview = item.clone();
    match state {
        State::MainType => {preview.kind = maintypes(option)?}
        State::SubType  => {let (_, kind) = subtypes(item.kind).into_iter().find(|(opt, _)| *opt == option)?;
                                preview.kind = kind;}
        State::Material => {preview.material = Material::from_repr(option as i8)?}
        State::Rarity   => {preview.rarity = option as u8}
        State::Model    => {if !model_seeds(item).contains(&option) { return None }
                                preview.seed = option;}
        State::Stat     => {preview.seed = stat_seed(item, option)?}
        State::Complete => return None
    }
    Some(preview)
}

fn item_validation(state: &mut State, item: &mut Item, player_level: i16) -> Result<(), String> {
    if DISABLED_ITEMS.contains(&item.kind) { return Err(format!("{} is currently disabled", item_name(item.kind))) }
    loop {
        match *state {
            State::MainType => return Ok(()),
            State::SubType  => {if subtypes(item.kind).is_empty() { *state = State::Material }
                                    else { return Ok(()) }}
            State::Material => {let valid_materials = materials::by_item_kind(item.kind);
                                    match valid_materials.len() {
                                        0 => return Err(format!("{} has no valid materials", item_name(item.kind))),
                                        1 => {item.material = valid_materials[0];
                                             *state = State::Rarity;}
                                        _ => return Ok(())}}
            State::Rarity   => {if item.kind.uses_rarity() { return Ok(()) }
                                    item.rarity = NORMAL;
                                    *state = State::Model;}
            State::Model    => {item.level = item.kind.item_level(player_level);
                                    if item.uses_models() { return Ok(()) }
                                    item.seed = 0;
                                    *state = State::Stat;}
            State::Stat     => {if item.kind.uses_stats() { return Ok(()) }
                                    *state = State::Complete;}
            State::Complete => return Ok(())
        }
    }
}

fn item_name(kind: Kind) -> String {
    let subtype = subtype_names(kind);
    if subtype.is_empty() { kind.to_string() } else { subtype }
}

fn subtype_names(kind: Kind) -> String {
    match kind {
        Kind::Consumable(consumable) => consumable.to_string(),
        Kind::Weapon(weapon)         => weapon.to_string(),
        Kind::Resource(resource)     => resource.to_string(),
        Kind::Candle(candle)         => candle.to_string(),
        Kind::Pet(race) |
        Kind::PetFood(race)          => race.to_string(),
        Kind::Quest(quest)           => quest.to_string(),
        Kind::Special(special)       => special.to_string(),
        _                            => String::new()
    }
}

const fn rarity_names(option: i32) -> &'static str {
    match option as u8 {
        NORMAL    => "Normal",
        UNCOMMON  => "Uncommon",
        RARE      => "Rare",
        EPIC      => "Epic",
        LEGENDARY => "Legendary",
        _         => ""
    }
}

fn npc_names(state: State, item: &Item, option: i32) -> String {
    match state {
        State::MainType => maintypes(option).map(|kind| kind.to_string()).unwrap_or_default(),
        State::SubType  => subtypes(item.kind)
            .into_iter()
            .find(|(opt, _)| *opt == option)
            .map(|(_, subtype)| subtype_names(subtype))
            .unwrap_or_default(),
        State::Material => Material::from_repr(option as i8).map(|material| material.to_string()).unwrap_or_default(),
        State::Rarity   => rarity_names(option).to_owned(),
        State::Model    => format!("Model\n{option}"),
        State::Stat     => item_preview(state, item, option)
            .map(|preview| preview.stats())
            .map(|stats| format!("C:{:.1}%\nT:{:.1}%", stats[Stat::Crit] * 100.0, stats[Stat::Tempo] * 100.0))
            .unwrap_or_default(),
        State::Complete => String::new()
    }
}