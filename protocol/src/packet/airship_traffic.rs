use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::packet::*;
use crate::ReadCwData;

impl<Readable: AsyncRead + Unpin> ReadCwData<AirshipTraffic> for Readable {
	async fn read_cw_data(&mut self) -> io::Result<AirshipTraffic> {
		Ok(AirshipTraffic {
			//explicit type annotation as a workaround for https://github.com/rust-lang/rust/issues/108362
			airships: ReadCwData::<Vec<Airship>>::read_cw_data(self).await?//self.read_cw_struct().await?
		})
	}
}

impl<Writable: AsyncWrite + Unpin> WriteCwData<AirshipTraffic> for Writable {
	async fn write_cw_data(&mut self, airship_traffic: &AirshipTraffic) -> io::Result<()> {
		self.write_cw_data(&airship_traffic.airships).await
	}
}


#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct Airship {
	pub id: i64,
	pub unknown_a: i32, //u8 + 3pad according to cuwo
	pub unknown_b: i32, //maybe padding
	pub position: Point3<i64>,
	pub velocity: Vector3<f32>,
	pub rotation: f32,
	pub station: Point3<i64>,
	pub path_rotation: f32,
	pub unknown_c: i32,//maybe padding
	pub destination: Point3<i64>,
	pub state: State,
	pub unknown_d: i32 //u8 + 3pad according to cuwo
}

impl<Readable: AsyncRead + Unpin> ReadCwData<Airship> for Readable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<Airship> for Writable {}

#[repr(i32)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum State {//from cuwo
	GoToStart,
	Landing,
	Takeoff,
	GoToDestination
}