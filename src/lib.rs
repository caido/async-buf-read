use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(feature = "tokio")]
use tokio::io::AsyncRead;

pub use self::buf_read_ext::AsyncPeek;
pub use self::buf_reader::AsyncBufReader;

mod buf_read_ext;
mod buf_reader;

pub trait AsyncBufRead: AsyncRead {
    fn poll_fill_buf(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        amt: usize,
    ) -> Poll<io::Result<&[u8]>>;

    fn consume(self: Pin<&mut Self>, amt: usize);
}
