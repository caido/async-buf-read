use std::ops::DerefMut;
use std::pin::Pin;
use std::task::{Context, Poll};

pub use self::buf_read_ext::AsyncBufReadExt;
pub use self::buf_reader::AsyncBufReader;
pub use self::passthrough::AsyncBufPassthrough;

mod buf_read_ext;
mod buf_reader;
mod io;
mod passthrough;
mod peek;

/// Reads bytes asynchronously and buffers them.
///
/// Utilities for working with `AsyncBufRead` values are provided by
/// [`AsyncBufReadExt`].
///
/// [`AsyncBufReadExt`]: crate::AsyncBufReadExt
pub trait AsyncBufRead: io::AsyncRead {
    /// Returns true if the inner reader has reached EOF.
    fn eof(self: Pin<&Self>) -> bool;

    /// Returns a slice of the internal buffer.
    fn buf(self: Pin<&Self>) -> &[u8];

    /// Attempts to return the contents of the internal buffer, filling it with more
    /// data from the inner reader if it less than the requested amount.
    ///
    /// On success, returns `Poll::Ready(Ok(buf))`.
    ///
    /// If no data is available for reading, the method returns
    /// `Poll::Pending` and arranges for the current task (via
    /// `cx.waker().wake_by_ref()`) to receive a notification when the object becomes
    /// readable or is closed.
    ///
    /// This function doesn't consume the data, it only returns a slice up
    /// to the requested amount. This means that subsequent calls to [`poll_read`]
    /// will return the same contents. As such, [`consume`] can be called
    /// with the number of bytes that are consumed from this buffer to
    /// ensure that the bytes are not returned by [`poll_read`].
    ///
    /// To check if the inner reader has reached EOF, use [`eof`].
    ///
    /// [`poll_read`]: AsyncRead::poll_read
    /// [`consume`]: AsyncBufRead::consume
    /// [`eof`]: AsyncBufRead::eof
    fn poll_fill_buf<'a>(
        self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
        amt: usize,
    ) -> Poll<io::Result<&'a [u8]>>;

    /// Tells this buffer that `amt` bytes have been consumed from the buffer,
    /// so they should no longer be returned in calls to [`poll_read`].
    ///
    /// This function is a lower-level call. It needs to be paired with the
    /// [`poll_fill_buf`] method to function properly. This function does
    /// not perform any I/O, it simply informs this object that some amount of
    /// its buffer, returned from [`poll_fill_buf`], has been consumed and should
    /// no longer be returned. As such, this function may do odd things if
    /// [`poll_fill_buf`] isn't called before calling it.
    ///
    /// The `amt` must be `<=` the number of bytes in the buffer returned by
    /// [`poll_fill_buf`].
    ///
    /// [`poll_read`]: AsyncRead::poll_read
    /// [`poll_fill_buf`]: AsyncBufRead::poll_fill_buf
    fn consume(self: Pin<&mut Self>, amt: usize);
}

macro_rules! deref_async_buf_read {
    () => {
        fn eof(self: Pin<&Self>) -> bool {
            Pin::new(&**self.get_ref()).eof()
        }

        fn buf(self: Pin<&Self>) -> &[u8] {
            Pin::new(&**self.get_ref()).buf()
        }

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
    fn eof(self: Pin<&Self>) -> bool {
        self.get_ref().as_ref().eof()
    }

    fn buf(self: Pin<&Self>) -> &[u8] {
        self.get_ref().as_ref().buf()
    }

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
    fn eof(self: Pin<&Self>) -> bool {
        false
    }

    fn buf(self: Pin<&Self>) -> &[u8] {
        self.get_ref()
    }

    fn poll_fill_buf(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        amt: usize,
    ) -> Poll<io::Result<&[u8]>> {
        let amt = std::cmp::min(self.len(), amt);
        Poll::Ready(Ok(&self[..amt]))
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        *self = &self[amt..];
    }
}
