/// A trait to set passthrough on nested buffered readers.
///
/// This allows for disabling the buffer even if the reader is
/// nested other buffered readers.
pub trait AsyncBufPassthrough {
    /// Set the buffered reader to start or stop acting as a passthrough for the
    /// underlying reader.
    ///
    /// When the buffer is in passthrough mode, it will not buffer any additional data.
    /// It will first exhaust the current buffer and then use the underlying reader to
    /// read more data afterward.
    ///
    /// This is useful when you need to peek data only at the beginning of a stream
    /// to determine how to parse it, but then don't need to buffer the rest of the stream.
    fn passthrough(&mut self, passthrough: bool);
}

#[cfg(feature = "tokio-rustls")]
impl<IO: AsyncBufPassthrough> AsyncBufPassthrough for tokio_rustls::server::TlsStream<IO> {
    fn passthrough(&mut self, passthrough: bool) {
        self.get_mut().0.passthrough(passthrough);
    }
}

#[cfg(feature = "tokio-rustls")]
impl<IO: AsyncBufPassthrough> AsyncBufPassthrough for tokio_rustls::client::TlsStream<IO> {
    fn passthrough(&mut self, passthrough: bool) {
        self.get_mut().0.passthrough(passthrough);
    }
}

#[cfg(feature = "tokio-openssl")]
impl<IO: AsyncBufPassthrough> AsyncBufPassthrough for tokio_openssl::SslStream<IO> {
    fn passthrough(&mut self, passthrough: bool) {
        self.get_mut().passthrough(passthrough);
    }
}
