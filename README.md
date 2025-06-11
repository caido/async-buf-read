# AsyncBufRead

[<img alt="github" src="https://img.shields.io/badge/github-caido/async-buf-read-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/caido/async-buf-read)
[<img alt="crates.io" src="https://img.shields.io/crates/v/async-buf-read?color=fc8d62&logo=rust&style=for-the-badge" height="20">](https://crates.io/crates/async-buf-read)

The goal of the library is to provide a better `AsyncBufRead` interface (and corresponding `AsyncBufReader`) than what is available in the runtimes like `tokio`.

## Why?

One the major limitation is that `BufReader` will never go beyond the allocated buffer, this makes it tricky for parser that might not know in advance how much space they will need. This implementation allows the buffer to grow infinitely based on the requested amount of data.

Another limitation is that the current interface force an often uncessary copy of the data from the buffer when read by the consumer.
This implementation allows peeking and consuming without a copy of data.

Finally, the current interface doesn't easily allow transforming the buffered reading in a passthrough.
This implementation gives you the flexibility to toggle on and off the buffering based on the application needs.

## Usage

```rust

```

## Other libraries

If you are doing sync IO, check [buffered-reader](https://crates.io/crates/buffered-reader).
