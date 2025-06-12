pub use std::io::Result;

#[cfg(feature = "tokio")]
pub(crate) use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
