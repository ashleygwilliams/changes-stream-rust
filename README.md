# changes-stream-rust

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
