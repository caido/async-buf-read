use async_buf_read::{AsyncBufPassthrough, AsyncBufReadExt, AsyncBufReader};
use tokio::io::AsyncReadExt;

#[tokio::test]
async fn test_buf_reader_read_basic() {
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

#[tokio::test]
async fn test_buf_reader_read_expand() {
    let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4, 11, 10, 9, 8, 13, 12, 14, 15];
    let mut reader = AsyncBufReader::with_chunk_size(5, inner);

    let mut buf = [0, 0, 0, 0];
    let nread = reader.read(&mut buf).await.unwrap();
    assert_eq!(nread, 4);
    assert_eq!(buf, [5, 6, 7, 0]);
    assert_eq!(reader.capacity(), 1);
    assert_eq!(reader.buffer(), [1]);

    let mut buf = [0, 0, 0, 0];
    let nread = reader.read(&mut buf).await.unwrap();
    assert_eq!(nread, 4);
    assert_eq!(buf, [1, 2, 3, 4]);
    assert_eq!(reader.capacity(), 2);
    assert_eq!(reader.buffer(), [11, 10]);

    let mut buf = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let nread = reader.read(&mut buf).await.unwrap();
    assert_eq!(nread, 8);
    assert_eq!(buf, [11, 10, 9, 8, 13, 12, 14, 15, 0, 0, 0, 0]);
    assert_eq!(reader.capacity(), 0);
    assert_eq!(reader.buffer(), []);

    assert_eq!(reader.read(&mut buf).await.unwrap(), 0);
}

#[tokio::test]
async fn test_buf_reader_peek() {
    let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4, 11, 10];
    let mut reader = AsyncBufReader::with_chunk_size(5, inner);

    let buf = reader.peek(4).await.unwrap();
    assert_eq!(buf, [5, 6, 7, 0]);
    assert_eq!(reader.capacity(), 5);
    assert_eq!(reader.buffer(), [5, 6, 7, 0, 1]);

    let buf = reader.peek(8).await.unwrap();
    assert_eq!(buf, [5, 6, 7, 0, 1, 2, 3, 4]);
    assert_eq!(reader.capacity(), 10);
    assert_eq!(reader.buffer(), [5, 6, 7, 0, 1, 2, 3, 4, 11, 10]);

    let buf = reader.peek(12).await.unwrap();
    assert_eq!(buf, [5, 6, 7, 0, 1, 2, 3, 4, 11, 10]);
    assert_eq!(reader.capacity(), 20);
    assert_eq!(reader.buffer(), [5, 6, 7, 0, 1, 2, 3, 4, 11, 10]);

    let buf = reader.peek(24).await.unwrap();
    assert_eq!(buf, [5, 6, 7, 0, 1, 2, 3, 4, 11, 10]);
    assert_eq!(reader.capacity(), 20);
    assert_eq!(reader.buffer(), [5, 6, 7, 0, 1, 2, 3, 4, 11, 10]);
}

#[tokio::test]
async fn test_buf_reader_passthrough() {
    let inner: &[u8] = &[5, 6, 7, 0, 1, 2, 3, 4, 11, 10];
    let mut reader = AsyncBufReader::with_chunk_size(5, inner);

    let buf = reader.peek(4).await.unwrap();
    assert_eq!(buf, [5, 6, 7, 0]);
    assert_eq!(reader.capacity(), 5);
    assert_eq!(reader.buffer(), [5, 6, 7, 0, 1]);

    reader.passthrough(true);

    let mut buf = [0, 0, 0, 0, 0, 0, 0, 0];
    let nread = reader.read(&mut buf).await.unwrap();
    assert_eq!(nread, 8);
    assert_eq!(buf, [5, 6, 7, 0, 1, 2, 3, 4]);
    assert_eq!(reader.capacity(), 0);
    assert_eq!(reader.buffer(), []);

    let buf = reader.peek(4).await.unwrap();
    assert_eq!(buf, []);
    assert_eq!(reader.capacity(), 0);
    assert_eq!(reader.buffer(), []);
}
