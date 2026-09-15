use protocol::nalgebra::Point2;
use protocol::packet::{AreaRequest, WorldUpdate};
use protocol::packet::area_request::Region;
use protocol::packet::common::Race;
use protocol::packet::world_update::Mission;
use protocol::packet::world_update::mission::{Objective, State};
use protocol::utils::constants::{SIZE_REGION, SIZE_SECTOR};

use crate::server::handle_packet::HandlePacket;
use crate::server::player::Player;
use crate::server::Server;

const SECTORS_PER_REGION: i32 = (SIZE_REGION / SIZE_SECTOR) as i32;

// client sends region requests every second until it is answered by the server
// vanilla server answers with the region's 64 mission slots; usually with:
// 60 RemoveMission, 2 Monster, and 2 Dungeon - berld sends 64 RemoveMission
impl HandlePacket<AreaRequest<Region>> for Server {
	async fn handle_packet(&self, source: &Player, packet: AreaRequest<Region>) {
		source.send_ignoring(&WorldUpdate::from(empty_missions(packet.0))).await;
	}
}

// todo: make this usable for events such as KotH
fn empty_missions(region: Point2<i32>) -> Vec<Mission> {
	let origin = region * SECTORS_PER_REGION;

	(0..SECTORS_PER_REGION * SECTORS_PER_REGION)
		.map(|index| Mission {
			sector: Point2::new(origin.x + index / SECTORS_PER_REGION, origin.y + index % SECTORS_PER_REGION),
			unknown_a: 0,
			unknown_b: 0,
			unknown_c: 0,
			id: 0,
			objective: Objective::RemoveMission,
			race: Race::default(),
			level: 0,
			rarity: 0,
			state: State::Ready,
			progress_current: 0,
			progress_maximum: 0,
			zone: Point2::origin()
		})
		.collect()
}