use std::{
    io,
    ops::DerefMut,
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(feature = "tokio")]
use tokio::io::AsyncRead;

pub use self::buf_read_ext::AsyncBufReadExt;
pub use self::buf_reader::AsyncBufReader;

mod buf_read_ext;
mod buf_reader;
mod peek;

pub trait AsyncBufRead: AsyncRead {
    fn poll_fill_buf<'a>(
        self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
        amt: usize,
    ) -> Poll<io::Result<&'a [u8]>>;

    fn consume(self: Pin<&mut Self>, amt: usize);
}

macro_rules! deref_async_buf_read {
    () => {
        fn poll_fill_buf(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            amt: usize,
        ) -> Poll<io::Result<&[u8]>> {
            Pin::new(&mut **self.get_mut()).poll_fill_buf(cx, amt)
        }

        fn consume(mut self: Pin<&mut Self>, amt: usize) {
            Pin::new(&mut **self).consume(amt)
        }
    };
}

impl<T: ?Sized + AsyncBufRead + Unpin> AsyncBufRead for Box<T> {
    deref_async_buf_read!();
}

impl<T: ?Sized + AsyncBufRead + Unpin> AsyncBufRead for &mut T {
    deref_async_buf_read!();
}

impl<P> AsyncBufRead for Pin<P>
where
    P: DerefMut + Unpin,
    P::Target: AsyncBufRead,
{
    fn poll_fill_buf(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        amt: usize,
    ) -> Poll<io::Result<&[u8]>> {
        self.get_mut().as_mut().poll_fill_buf(cx, amt)
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        self.get_mut().as_mut().consume(amt);
    }
}

impl AsyncBufRead for &[u8] {
    fn poll_fill_buf(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _amt: usize,
    ) -> Poll<io::Result<&[u8]>> {
        Poll::Ready(Ok(*self))
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        *self = &self[amt..];
    }
}
