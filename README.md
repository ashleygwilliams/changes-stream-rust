# changes-stream-rust

[![travis badge](https://travis-ci.org/ashleygwilliams/changes-stream-rust.svg?branch=master)](https://travis-ci.org/ashleygwilliams/changes-stream-rust)


an implementation of [`changes-stream`](https://github.com/jcrugzz/changes-stream) in Rust.

this code reads in a readable stream from an endpoint and returns each chunk in JSON.


## usage

in your `Cargo.toml`:

```toml
[dependencies]
changes-stream = "0.2"
```

from [examples/follower.rs](/examples/follower.rs):

```rust
use changes_stream::ChangesStream;

#[tokio::main]
async fn main() {
    let url = String::from("https://replicate.npmjs.com/_changes");
    let mut changes = ChangesStream::new(url);
    changes.on(|change| {
        println!("{}: {}", change.seq, change.id);
    });
    changes.run().await;
}
```
