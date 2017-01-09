# changes-stream-rust

[![travis badge](https://travis-ci.org/ashleygwilliams/changes-stream-rust.svg?branch=master)](https://travis-ci.org/ashleygwilliams/changes-stream-rust)


an implementation of [`changes-stream`](https://github.com/jcrugzz/changes-stream) in Rust.

this code reads in a readable stream from an endpoint and returns each chunk in JSON.

this code works off of the [`tokio` branch] of [`hyper`] to take advantage of new Rust Futures.

[`tokio` branch]: https://github.com/hyperium/hyper/tree/tokio
[`hyper`]: https:///crates.io/crates/hyper

`changes-stream-rust` only works on nightly because it uses [`serde`].

[`serde`]: https://crates.io/crates/serde

## usage

in your `Cargo.toml`:

```toml
[dependencies]
changes-stream = { git = "https://github.com/ashleygwilliams/changes-stream-rust.git" }
```

from [examples/follower.rs](/examples/follower.rs):

```rust
extern crate changes_stream;
extern crate futures;

use std::io;
use std::io::Write;

use changes_stream::ChangesStream;

fn main() {
    let url = "https://replicate.npmjs.com/_changes".to_string();
    let mut changes = ChangesStream::new(url);

    changes.on(|change| {
        io::stdout().write_all(&change).unwrap();
    });

    changes.run();
}
```
