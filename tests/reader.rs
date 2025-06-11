use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};

use async_buf_read::AsyncBufReader;
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};

struct MaybePending<'a> {
    inner: &'a [u8],
    ready_read: bool,
    ready_fill_buf: bool,
}

impl<'a> MaybePending<'a> {
    fn new(inner: &'a [u8]) -> Self {
        Self {
            inner,
            ready_read: false,
            ready_fill_buf: false,
        }
    }
}

impl AsyncRead for MaybePending<'_> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        if self.ready_read {
            self.ready_read = false;
            Pin::new(&mut self.inner).poll_read(cx, buf)
        } else {
            self.ready_read = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[tokio::test]
async fn test_buffered_reader_basic() {
    let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4];
    let mut reader = AsyncBufReader::with_chunk_size(3, inner);

    let mut buf = [0, 0, 0];
    let nread = reader.read(&mut buf).await.unwrap();
    assert_eq!(nread, 3);
    assert_eq!(buf, [5, 6, 7]);
    assert_eq!(reader.buffer(), []);

    let mut buf = [0, 0];
    let nread = reader.read(&mut buf).await.unwrap();
    assert_eq!(nread, 2);
    assert_eq!(buf, [0, 1]);
    assert_eq!(reader.buffer(), [2]);

    let mut buf = [0];
    let nread = reader.read(&mut buf).await.unwrap();
    assert_eq!(nread, 1);
    assert_eq!(buf, [2]);
    assert_eq!(reader.buffer(), []);

    let mut buf = [0, 0, 0];
    let nread = reader.read(&mut buf).await.unwrap();
    assert_eq!(nread, 2);
    assert_eq!(buf, [3, 4, 0]);
    assert_eq!(reader.buffer(), []);

    assert_eq!(reader.read(&mut buf).await.unwrap(), 0);
}
