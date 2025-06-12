use std::{
    io::{self, IoSlice},
    pin::Pin,
    task::{Context, Poll, ready},
};

use bytes::{Buf, BufMut, BytesMut};
use pin_project_lite::pin_project;

use crate::io::{AsyncRead, AsyncWrite, ReadBuf};
use crate::{AsyncBufPassthrough, AsyncBufRead};

pin_project! {
    /// The `AsyncBufReader` struct adds buffering to any reader.
    ///
    /// This allows for both efficient reading of small amounts of data and
    /// peeking for parsing were you will want to read the data multiple times.
    /// The internal buffer of `AsyncBufReader` will expand based on the requested
    /// amount of data.
    ///
    /// When the `AsyncBufReader` is dropped, the contents of its buffer will be
    /// discarded. Creating multiple instances of a `AsyncBufReader` on the same
    /// stream can cause data loss.
    pub struct AsyncBufReader<R> {
        #[pin]
        reader: R,
        passthrough: bool,
        buf: BytesMut,
        chunk_size: usize,
        eof: bool,
    }
}

const DEFAULT_CHUNK_SIZE: usize = 8 * 1024;

impl<R: AsyncRead> AsyncBufReader<R> {
    /// Creates a new `AsyncBufReader` with the default chunk size.
    pub fn new(reader: R) -> Self {
        Self::with_chunk_size(DEFAULT_CHUNK_SIZE, reader)
    }

    /// Creates a new `AsyncBufReader` with the given chunk size.
    ///
    /// The chunk size is a hint for the amount of data that will be read into the buffer at once.
    pub fn with_chunk_size(chunk_size: usize, reader: R) -> Self {
        Self {
            reader,
            buf: BytesMut::with_capacity(chunk_size),
            passthrough: false,
            chunk_size,
            eof: false,
        }
    }

    /// Returns the current capacity of the internal buffer.
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }

    /// Returns the current length of the internal buffer.
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    /// Returns true if the internal buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    /// Gets a reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    pub fn get_ref(&self) -> &R {
        &self.reader
    }

    /// Gets a pinned mutable reference to the underlying reader.
    ///
    /// It is inadvisable to directly read from the underlying reader.
    pub fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut R> {
        self.project().reader
    }

    /// Returns a reference to the internally buffered data.
    pub fn buffer(&self) -> &[u8] {
        self.buf.as_ref()
    }

    /// Invalidates all data in the internal buffer.
    #[inline]
    fn discard_buffer(self: Pin<&mut Self>) {
        let me = self.project();
        // Force drop the buffer to ensure the memory is freed
        *me.buf = BytesMut::new();
    }
}

impl<R> AsyncBufPassthrough for AsyncBufReader<R> {
    fn passthrough(&mut self, enabled: bool) {
        self.passthrough = enabled;
    }
}

impl<R: AsyncRead> AsyncRead for AsyncBufReader<R> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        // In passthrough mode or if the requested amount of data is greater than the chunk size,
        // empty the buffer then pass through the read to the underlying reader.
        if self.passthrough || buf.remaining() >= self.chunk_size {
            if !self.buf.is_empty() {
                let amt = std::cmp::min(buf.remaining(), self.buf.len());
                buf.put_slice(&self.buf[..amt]);
                self.as_mut().consume(amt);
                if self.buf.is_empty() && self.passthrough {
                    self.as_mut().discard_buffer();
                }
                if buf.remaining() == 0 {
                    return Poll::Ready(Ok(()));
                }
            }
            if self.eof {
                return Poll::Ready(Ok(()));
            }
            return self.get_pin_mut().poll_read(cx, buf);
        }
        let rem = ready!(self.as_mut().poll_fill_buf(cx, buf.remaining()))?;
        let amt = std::cmp::min(rem.len(), buf.remaining());
        buf.put_slice(&rem[..amt]);
        self.consume(amt);
        Poll::Ready(Ok(()))
    }
}

impl<R: AsyncRead> AsyncBufRead for AsyncBufReader<R> {
    fn eof(self: Pin<&Self>) -> bool {
        self.get_ref().eof
    }

    fn buf(self: Pin<&Self>) -> &[u8] {
        self.get_ref().buffer()
    }

    fn poll_fill_buf<'a>(
        self: Pin<&'a mut Self>,
        cx: &mut Context<'_>,
        amt: usize,
    ) -> Poll<io::Result<&'a [u8]>> {
        let me = self.project();

        // If we are in passthrough mode or at EOF, return the buffer.
        // Don't attempt to fill the buffer with more data.
        if *me.passthrough || *me.eof {
            let rem = std::cmp::min(amt, me.buf.len());
            return Poll::Ready(Ok(&me.buf[..rem]));
        }

        // If the buffer has enough data, return it
        if me.buf.len() >= amt {
            return Poll::Ready(Ok(&me.buf[..amt]));
        } else {
            // Check if we have enough space in the buffer
            if me.buf.capacity() < amt {
                me.buf
                    .reserve(std::cmp::max(*me.chunk_size, amt - me.buf.len()));
            }
        }

        let mut buf = ReadBuf::uninit(me.buf.spare_capacity_mut());
        ready!(me.reader.poll_read(cx, &mut buf))?;
        let n = buf.filled().len();
        if n == 0 {
            *me.eof = true;
        }
        unsafe {
            // SAFETY: We know that filled will be at maximum the spared capacity and
            // won't exceed the buffer's capacity
            me.buf.advance_mut(n);
        };

        let rem = std::cmp::min(amt, me.buf.len());
        Poll::Ready(Ok(&me.buf[..rem]))
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        let me = self.project();
        me.buf.advance(amt);
    }
}

impl<R: AsyncRead + AsyncWrite> AsyncWrite for AsyncBufReader<R> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.get_pin_mut().poll_write(cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        self.get_pin_mut().poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.get_ref().is_write_vectored()
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.get_pin_mut().poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.get_pin_mut().poll_shutdown(cx)
    }
}
