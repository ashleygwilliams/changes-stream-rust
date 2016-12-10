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
