# changes-stream-rust

an implementation of [`changes-stream`](https://github.com/jcrugzz/changes-stream) in Rust.
this code works off of the `tokio` branch of hyper to take advantage of new Rust Futures.

## usage

```
cargo run -- <url>
```

**NOTE** this does not currently support https, working on integrating TLS into hyper rn.
