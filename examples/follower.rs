extern crate pretty_env_logger;
extern crate changes_stream;
extern crate futures;

use std::io;
use std::io::Write;

use changes_stream::ChangesStream;

use std::env;

fn main() {
    pretty_env_logger::init();

    let url = match env::args().nth(1) {
        Some(url) => url,
        None => {
            println!("Usage: client <url>");
            return;
        }
    };

    let mut changes = ChangesStream::new(url);

    changes.on(|change| {
        io::stdout().write_all(&change).unwrap();
    });

    changes.run();
}
