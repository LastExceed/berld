use async_trait::async_trait;
use tokio::io;

use protocol::nalgebra::Vector3;
use protocol::packet::{CreatureAction, WorldUpdate};
use protocol::packet::creature_action::CreatureActionType;
use protocol::packet::world_update::{Pickup, SoundEffect};
use protocol::packet::world_update::sound_effect::Sound;
use protocol::utils::constants::SIZE_BLOCK;
use protocol::utils::sound_position_of;

use crate::handle_packet::HandlePacket;
use crate::player::Player;
use crate::server::Server;

#[async_trait]
impl HandlePacket<CreatureAction> for Server {
	async fn handle_packet(&self, source: &Player, packet: CreatureAction) -> io::Result<()> {
		match packet.type_ {
			CreatureActionType::Bomb => {
				source.notify("bombs are disabled").await;

				//the player consumed a bomb, so we need to reimburse it
				source.send_ignoring(&WorldUpdate {
					pickups: vec![Pickup {
						interactor: source.creature.read().await.id,
						item: packet.item
					}],
					..Default::default()
				}).await;
			}
			CreatureActionType::Talk => {
				source.notify("quests coming soon(tm)").await;
			}
			CreatureActionType::ObjectInteraction => {
				source.notify("object interactions are disabled").await;
			}
			CreatureActionType::PickUp => {
				if let Some(item) = self.remove_drop(packet.zone, packet.item_index as usize).await {
					source.send_ignoring(&WorldUpdate {
						pickups: vec![Pickup {
							interactor: source.creature.read().await.id,
							item
						}],
						sound_effects: vec![
							SoundEffect {
								position: sound_position_of(source.creature.read().await.position),
								sound: Sound::Pickup,
								pitch: 1f32,
								volume: 1f32
							}
						],
						..Default::default()
					}).await;
				}
			}
			CreatureActionType::Drop => {
				let creature_guard = source.creature.read().await;

				self.add_drop(
					packet.item,
					creature_guard.position - Vector3::new(0, 0, SIZE_BLOCK),
					creature_guard.rotation.yaw
				).await;
			}
			CreatureActionType::CallPet => {
				//source.notify("pets are disabled".to_owned());
			}
		}

		Ok(())
	}
}