# AsyncBufRead

[<img alt="github" src="https://img.shields.io/badge/github-caido/async-buf-read-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/caido/async-buf-read)
[<img alt="crates.io" src="https://img.shields.io/crates/v/async-buf-read?color=fc8d62&logo=rust&style=for-the-badge" height="20">](https://crates.io/crates/async-buf-read)

The goal of the library is to provide a better interface for `AsyncBufReader` than what is available in the runtime like `tokio`.

## Why?

One the major limitation is that `BufReader` will never go beyond the allocated buffer, this makes it tricky for parser that might not know in advance how much space they will need.

Another limitation is that the current interface force an often uncessary copy of the data from the buffer when read by the consumer.

## Usage

```rust

```
