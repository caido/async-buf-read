use crate::AsyncBufRead;
use crate::peek::{Peek, peek};

/// An extension trait which adds utility methods to [`AsyncBufRead`] types.
///
/// [`AsyncBufRead`]: crate::AsyncBufRead
pub trait AsyncBufReadExt: AsyncBufRead {
    /// Returns true if the inner reader has reached EOF.
    fn ended(&self) -> bool
    where
        Self: Unpin,
    {
        std::pin::Pin::new(self).eof()
    }

    /// Returns a slice of the internal buffer.
    fn buffer(&self) -> &[u8]
    where
        Self: Unpin,
    {
        std::pin::Pin::new(self).buf()
    }

    /// Peek into the content of the internal buffer, filling it with more
    /// data from the inner reader if it less than the requested amount.
    ///
    /// This function doesn't consume the data, it only returns a slice up
    /// to the requested amount. This means that subsequent calls to `read`
    /// will return the same contents. As such, [`consume`] can be called
    /// with the number of bytes that are consumed from this buffer to
    /// ensure that the bytes are not returned by `read`.
    ///
    /// To check if the inner reader has reached EOF, use [`ended`].
    ///
    /// Equivalent to:
    ///
    /// ```ignore
    /// async fn peek(&mut self, amt: usize) -> io::Result<&[u8]>;
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an I/O error if the underlying reader was
    /// read, but returned an error.
    ///
    /// # Cancel safety
    ///
    /// This method is cancel safe. If you use it as the event in a
    /// `select!` statement and some other branch completes first,
    /// then it is guaranteed that no data was read.
    ///
    /// [`consume`]: crate::AsyncBufReadExt::consume
    /// [`ended`]: crate::AsyncBufReadExt::ended
    fn peek(&mut self, amt: usize) -> Peek<'_, Self>
    where
        Self: Unpin,
    {
        peek(self, amt)
    }

    /// Tells this buffer that `amt` bytes have been consumed from the
    /// buffer, so they should no longer be returned in calls to read.
    ///
    /// The `amt` must be less than the number of bytes in the buffer
    /// returned by [`peek`].
    ///
    /// [`peek`]: crate::AsyncBufReadExt::peek
    fn consume(&mut self, amt: usize)
    where
        Self: Unpin,
    {
        std::pin::Pin::new(self).consume(amt);
    }
}

impl<R: AsyncBufRead + ?Sized> AsyncBufReadExt for R {}
