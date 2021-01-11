# changes-stream-rust

[![travis badge](https://travis-ci.org/ashleygwilliams/changes-stream-rust.svg?branch=master)](https://travis-ci.org/ashleygwilliams/changes-stream-rust)


an implementation of [`changes-stream`](https://github.com/jcrugzz/changes-stream) in Rust.

this code reads in a readable stream from an endpoint, parses each line and returns CouchDB changes events as defined in [src/event.rs](/src/event.rs).


## usage

in your `Cargo.toml`:

```toml
[dependencies]
changes-stream = "0.2"
```

from [examples/follower.rs](/examples/follower.rs):

```rust
use changes_stream::{ChangesStream, Event};
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
