# changes_stream2

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](
https://github.com/elwerene/changes-stream-rust/blob/master/LICENSE)
[![Cargo](https://img.shields.io/crates/v/changes-stream2.svg)](
https://crates.io/crates/changes-stream2)
[![Documentation](https://docs.rs/changes-stream2/badge.svg)](
https://docs.rs/changes-stream2)

Fork of https://github.com/ashleygwilliams/changes-stream-rust / https://crates.io/crates/changes-stream.

An implementation of [`changes-stream`](https://github.com/jcrugzz/changes-stream) in Rust.

This code reads in a readable stream from an endpoint, parses each line and returns CouchDB changes events as defined in [src/event.rs](/src/event.rs).


## usage

in your `Cargo.toml`:

```toml
[dependencies]
changes-stream2 = "0.2"
```

from [examples/follower.rs](/examples/follower.rs):

```rust
use changes_stream2::{ChangesStream, Event};
use futures_util::stream::StreamExt;

#[tokio::main]
async fn main() {
    let url = "https://replicate.npmjs.com/_changes".to_string();
    let mut changes = ChangesStream::new(url).await.unwrap();
    while let Some(event) = changes.next().await {
        match event {
            Ok(Event::Change(change)) => println!("Change ({}): {}", change.seq, change.id),
            Ok(Event::Finished(finished)) => println!("Finished: {}", finished.last_seq),
            Err(err) => println!("Error: {:?}", err),
        }
    }
}
```
