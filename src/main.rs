extern crate pretty_env_logger;
extern crate changes_stream;

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

    let changes = ChangesStream::new(url);
    changes.run();
}
