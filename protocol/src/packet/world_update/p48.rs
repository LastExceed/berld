use tokio::io::{AsyncRead, AsyncWrite};

use crate::{ReadCwData, WriteCwData};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct P48sub(pub [u8; 16]);

impl<Readable: AsyncRead + Unpin> ReadCwData<P48sub> for Readable {}
impl<Writable: AsyncWrite + Unpin> WriteCwData<P48sub> for Writable {}