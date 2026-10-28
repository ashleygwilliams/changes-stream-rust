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

## errors

A line that cannot be parsed yields `Err(Error::ParsingFailed)` and the stream continues. A failed read, such as a lost connection or no data for 60 seconds, yields `Err(Error::RequestFailed)` once and then the stream ends. For a continuous feed, request a `heartbeat` shorter than 60 seconds, for example `?feed=continuous&heartbeat=30000`.

## features

### metrics

Enables metrics collection of the changes stream as counter values. Each counter carries a `database` label, generated from the host and path of the url. The metrics are:
 * `couchdb_changes_bytes_total`: Total bytes read from the changes stream
 * `couchdb_changes_entries_total`: Total parsed entries

### raw_value_doc

Changes the type of ChangeEvent::Doc from `serde_json::Map<String, serde_json::Value>` to `serde_json::value::RawValue`.

### native-tls

Use the native-tls crate for TLS connections. This is the default.

### rustls-tls

Use the rustls crate for TLS connections.