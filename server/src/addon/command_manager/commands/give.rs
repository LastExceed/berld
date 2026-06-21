use std::str::SplitWhitespace;

use protocol::packet::common::{item, Item};
use protocol::packet::world_update::Pickup;
use protocol::packet::WorldUpdate;
use protocol::utils::constants::materials::by_item_kind;
use protocol::utils::constants::rarity::*;
use protocol::utils::power_of;
use tap::Pipe;

use crate::addon::command_manager::{Command, CommandResult};
use crate::server::player::Player;
use crate::server::Server;

impl Command for super::Give {
    const LITERAL: &'static str = "give";
    const ADMIN_ONLY: bool = false;

    async fn execute<'fut>(&'fut self, _server: &'fut Server, caller: Option<&'fut Player>, params: &'fut mut SplitWhitespace<'fut>) -> CommandResult {
        let caller = caller.ok_or("ingame only")?;
        
        let mut item = Item::default();
        
        let param_1 = params.next().ok_or("usage: /give weapon.dagger level=500 tier=4 seed=6969 material=iron")?;
        

        let (input_kind, input_variant) = param_1.split_once('.').unwrap_or((param_1, ""));
    
        use item::Kind::*;
        item.kind = input_kind
            .parse::<item::Kind>()
            .map_err(|_| "unknown item type")?
            .pipe(|kind| match kind {
                Consumable(_) => input_variant.parse().map(Consumable),
                Weapon(_)     => input_variant.parse().map(Weapon    ),
                Resource(_)   => input_variant.parse().map(Resource  ),
                Candle(_)     => input_variant.parse().map(Candle    ),
                Pet(_)        => input_variant.parse().map(Pet       ),
                PetFood(_)    => input_variant.parse().map(PetFood   ),
                Quest(_)      => input_variant.parse().map(Quest     ),
                Special(_)    => input_variant.parse().map(Special   ),
                other => Ok(other)
            })
            .map_err(|_| "unknown item sub-type")?;

        let valid_materials = by_item_kind(item.kind);
        if item.kind.uses_rarity() {item.rarity = LEGENDARY} else {item.rarity = NORMAL}
        if item.kind.uses_level() {item.level = caller.character.read().await.level as i16} else {item.level = 1}
        if !valid_materials.is_empty() {item.material = valid_materials[0]}

        for param in params {
            if param == "adapted" {
                item.flags.set(item::Flag::Adapted, true);
                continue;
            }

            let(property, value) = param.split_once('=').ok_or("invalid parameter")?;
            
            match property {
                "seed"     => item.seed     = value.parse().map_err(|_| "invalid random/seed")?,
                "tier"     => item.rarity   = value.parse().map_err(|_| "invalid tier/rarity")?,
                "material" => item.material = value.parse().map_err(|_| "invalid material"   )?,
                "level"    => item.level    = value.parse().map_err(|_| "invalid level"      )?,
                _ => return Err("unknown property")
            }
        }

        if valid_materials.is_empty() {return Err("item has no valid materials")}
        if !valid_materials.contains(&item.material) {
            let materials_list = valid_materials
                .iter()
                .map(|m| format!("{:?}", m))
                .collect::<Vec<_>>()
                .join(", ");
            caller.notify(format!("Try: {}", materials_list)).await;
            return Err("incompatible material");
        }

        if !item.kind.uses_rarity() && item.rarity != NORMAL {return Err("item does not use rarity")}
        if !item.kind.uses_level() && item.level != 1 {return Err("item does not use level")}
        if !item.kind.uses_seed() && item.seed != 0 {return Err("item does not use seed")}
        
        (1..=power_of(500))
            .contains(&power_of(item.level as _))
            .ok_or("item level out of bounds")?;
        
        let pickup = Pickup {
            interactor: caller.id,
            item
        };
        caller.send_ignoring(&WorldUpdate::from(pickup)).await;
        
        Ok(None)
    }
}