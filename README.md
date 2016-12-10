# changes-stream-rust

[![travis badge](https://travis-ci.org/ashleygwilliams/changes-stream-rust.svg?branch=master)](https://travis-ci.org/ashleygwilliams/changes-stream-rust)


an implementation of [`changes-stream`](https://github.com/jcrugzz/changes-stream) in Rust.
this code works off of the [`tokio` branch of hyper] to take advantage of new Rust Futures.

[`tokio` branch of hyper]: https://github.com/hyperium/hyper/tree/tokio

## usage

in your `Cargo.toml`:

```toml
[dependencies]
changes-stream = { git = "https://github.com/ashleygwilliams/changes-stream-rust.git" }
```

from [examples/follower.rs](/examples/follower.rs):

```rust
let mut changes = ChangesStream::new(url);

changes.on(|change| {
    io::stdout().write_all(&change);
});

changes.run();
```

NOTE: due to a TLS issue on MacOS and OSX, `https` only works on Linux, currently.
