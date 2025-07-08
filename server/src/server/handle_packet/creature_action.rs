use protocol::nalgebra::Vector3;
use protocol::packet::common::item::Material::Obsidian;
use protocol::packet::common::Item;
use protocol::packet::{CreatureAction, WorldUpdate};
use protocol::packet::common::item::Kind::*;
use protocol::packet::creature_action::Kind::*;
use protocol::packet::world_update::{Pickup, sound, Sound};
use protocol::utils::constants::SIZE_BLOCK;

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

impl HandlePacket<CreatureAction> for Server {
	async fn handle_packet(&self, source: &Player, packet: CreatureAction) {
		match packet.kind {
			Bomb => {
				source.notify("bombs are disabled").await;

				//the player consumed a bomb, so we need to reimburse it
				let pickup = Pickup {
					interactor: source.id,
					item: packet.item
				};
				source.send_ignoring(&WorldUpdate::from(pickup)).await;
			}
			Talk => {
				source.notify("quests coming soon(tm)").await;
			}
			ObjectInteraction => {
				source.notify("object interactions are disabled").await;
			}
			PickUp => {
				#[expect(clippy::cast_sign_loss, reason = "TODO")] //todo: make item_index unsigned?
				let Some(item) = self.remove_drop(packet.zone, packet.item_index as usize).await
					else { return; };

				source.send_ignoring(&WorldUpdate {
					pickups: vec![Pickup { item, interactor: source.id }],
					sounds: vec![Sound::at(source.character.read().await.position, sound::Kind::Pickup)],
					..Default::default()
				}).await;
			}
			Drop => {
				if is_crash_item(&packet.item) {
					self.kick(source, "crash item dropped").await;
					return;
				}
				
				let character = source.character.read().await;
				let position = character.position - Vector3::new(0, 0, SIZE_BLOCK);
				let rotation = character.rotation.yaw;
				drop(character);

				self.add_drop(packet.item, position, rotation).await;
			}
			CallPet => {
				//source.notify("pets are disabled".to_owned());
			}
		}
	}
}

const fn is_crash_item(item: &Item) -> bool {
	matches!(
		item,
		Item { kind: Void, .. } |
		Item { kind: Block, material: Obsidian, .. }
	)
}