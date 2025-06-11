pub use std::io::Result;

#[cfg(feature = "tokio")]
#[allow(unused_imports)]
pub(crate) use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, ReadBuf};
