extern crate serde_json;
extern crate changes_stream;
extern crate futures;

use changes_stream::ChangesStream;

fn main() {
    let url = "https://replicate.npmjs.com/_changes".to_string();
    let mut changes = ChangesStream::new(url);

    changes.on(|change| {
        let data = serde_json::to_string(change).unwrap();
        println!("{}", data);
    });

    changes.run();
}
