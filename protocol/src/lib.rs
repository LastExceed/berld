#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(associated_type_defaults)]
#![allow(const_evaluatable_unchecked)]
#![feature(async_closure)]
#![feature(async_fn_in_trait)]

use std::mem::size_of;

pub use nalgebra;
pub use rgb;
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::packet::*;
use crate::utils::io_extensions::{ReadStruct, WriteStruct};

pub mod packet;
pub mod utils;

pub trait Packet {
	const ID: packet::Id;//dedicated type ensures this can't be used in arithmetic operations
}


pub trait ReadCwData<CwStruct>: AsyncRead + Unpin + Sized {
	async fn read_cw_data(&mut self) -> io::Result<CwStruct>
		where [(); size_of::<CwStruct>()]:
	{
		self.read_struct().await
	}
}

pub trait WriteCwData<CwStruct>: AsyncWrite + Unpin + Sized {
	async fn write_cw_data(&mut self, cw_data: &CwStruct) -> io::Result<()> {
		self.write_struct(cw_data).await
	}
}

//todo: use blanket default implementation, then specialize the rest (waiting for https://github.com/rust-lang/rust/issues/108309)
impl<Readable: AsyncRead + Unpin> ReadCwData<MultiCreatureUpdate > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<ServerTick          > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<IngameDatetime      > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<CreatureAction      > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<Hit                 > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<StatusEffect        > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<Projectile          > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<ZoneDiscovery       > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<RegionDiscovery     > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<MapSeed             > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<ConnectionAcceptance> for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<ProtocolVersion     > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<ConnectionRejection > for Readable {}

impl<Writable: AsyncWrite + Unpin> WriteCwData<MultiCreatureUpdate > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<ServerTick          > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<IngameDatetime      > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<CreatureAction      > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<Hit                 > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<StatusEffect        > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<Projectile          > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<ZoneDiscovery       > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<RegionDiscovery     > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<MapSeed             > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<ConnectionAcceptance> for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<ProtocolVersion     > for Writable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<ConnectionRejection > for Writable {}
//ChatMessageFromServer
//ChatMessageFromClient
//WorldUpdate                //which can be found in their respective module
//AirshipTraffic             //these packets have non-default trait implementations
//CreatureUpdate