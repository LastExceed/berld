#![expect(incomplete_features, reason = "generic_const_exprs is incomplete, but works for our purposes")]
#![feature(generic_const_exprs)]
#![feature(associated_type_defaults)]
#![feature(async_closure)]
#![feature(min_specialization)]
#![feature(lint_reasons)]

#![expect(async_fn_in_trait, reason = "TODO")] //TODO: investigate if AFIT desugaring could obsolete Unpin trait bounds

use std::io::ErrorKind::InvalidData;
use std::mem::size_of;

use boolinator::Boolinator;
pub use nalgebra;
pub use rgb;
use strum::IntoEnumIterator;
use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

use crate::packet::*;
use crate::utils::io_extensions::{ReadArbitrary, WriteArbitrary};

pub mod packet;
pub mod utils;
#[cfg(test)]
mod tests;

pub trait Packet //: where for<T: AsyncRead> T: ReadCwData<Self> //TODO: investigate if #![feature(non_lifetime_binders)] is usable yet
{
	const ID: packet::Id;//dedicated type ensures this can't be used in arithmetic operations
}


pub trait ReadCwData<CwStruct>: AsyncRead + Unpin + Sized {
	async fn read_cw_data(&mut self) -> io::Result<CwStruct>
		where [(); size_of::<CwStruct>()]:
	{
		self.read_arbitrary().await
	}
}

pub trait WriteCwData<CwStruct>: AsyncWrite + Unpin + Sized {
	async fn write_cw_data(&mut self, cw_data: &CwStruct) -> io::Result<()> {
		self.write_arbitrary(cw_data).await
	}
}

//todo: use blanket default implementation, then specialize the rest (waiting for https://github.com/rust-lang/rust/issues/108309)
impl<Readable: AsyncRead + Unpin> ReadCwData<MultiCreatureUpdate > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<ServerTick          > for Readable {}
impl<Readable: AsyncRead + Unpin> ReadCwData<IngameDatetime      > for Readable {}
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
//AirshipTraffic             //these packets have non-default trait implementations
//WorldUpdate                //which can be found in their respective module
//CreatureUpdate
//CreatureAction

//todo: this should honestly be done entirely with macros, else its gonna be a bunch of copypasta
struct Validator;

impl Validator {
	fn validate_enum<E: IntoEnumIterator + PartialEq>(e: &E) -> io::Result<()> {
		E::iter().any(|variant| *e == variant).ok_or(InvalidData.into())
	}
}

trait Validate<T> {
	fn validate(data: &T) -> io::Result<()>;
}

impl<T> Validate<T> for Validator {
	default fn validate(_data: &T) -> io::Result<()> {
		Ok(())
	}
}