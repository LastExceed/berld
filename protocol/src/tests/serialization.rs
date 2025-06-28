use std::fmt::Debug;

use crate::{Packet, ReadCwData, WriteCwData};
use crate::utils::io_extensions::{ReadPacket as _, WritePacket as _};

mod creature_update;
mod multi_creature_update;
mod server_tick;
mod airship_traffic;
mod world_update;
mod ingame_datetime;
mod creature_action;
mod hit;
mod status_effect;
mod projectile;
mod chat_message;
mod zone_discovery;
mod region_discovery;
mod mapseed;
mod connection_acceptance;
mod protocol_version;
mod connection_rejection;

async fn test_deserialization<P : Packet + PartialEq + Debug, const SIZE: usize>(bytes: [u8; SIZE], packet: P)
	where
		[(); size_of::<P>()]:,
		for<'data> &'data [u8] : ReadCwData<P> //todo: this bound shouldn't be necessary. further restrict Packet to imply this by default
{
	assert_eq!(
		bytes
			.as_slice()
			.read_packet::<P>()
			.await
			.unwrap(),
		packet
	);
}

async fn test_serialization<P : Packet + PartialEq + Debug>(packet: P)
	where Vec<u8> : WriteCwData<P>,
	      [(); size_of::<P>()]:,
	      for<'data> &'data [u8] : ReadCwData<P>
{
	let mut buffer = Vec::new();
	buffer.write_packet(&packet).await.unwrap();
	let re_deserialized = (&buffer[4..]).read_packet().await.unwrap();

	assert_eq!(packet, re_deserialized); //todo: skipping id bytes smells
}

///////////////////

///macro is necessary for splitting tasks into separate tests
#[expect(clippy::crate_in_macro_def, reason = "can't remember lol")]
#[macro_export]
macro_rules! generate_serialization_tests {
	($packet:expr, $bytes:expr) => {
		use crate::tests::serialization::{test_deserialization, test_serialization};
		use crate::packet::*;

		#[tokio::test]
		async fn deserialize() {
			test_deserialization(
				$bytes,
				$packet
			).await;
		}

		#[tokio::test]
		async fn re_deserialize() {
			test_serialization(
				$packet
			).await;
		}
	}
}