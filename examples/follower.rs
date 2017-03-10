extern crate serde_json;
extern crate changes_stream;
extern crate futures;

use changes_stream::ChangesStream;

fn main() {
    let url = String::from("https://replicate.npmjs.com/_changes");
    let mut changes = ChangesStream::new(url);
    changes.on(|change| {
        println!("{}: {}", change.seq, change.id);
    });
    changes.run();
}
