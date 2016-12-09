#![deny(warnings)]
extern crate futures;
extern crate hyper;
extern crate tokio_core;

use futures::Future;
use futures::stream::Stream;

use hyper::Client;

use std::io::{self, Write};

pub struct ChangesStream {
  db: String,
}

impl ChangesStream {
  pub fn new(db: String) -> ChangesStream {
    ChangesStream {
      db: db,
    }
  }

  pub fn db(&self) -> &String {
    &self.db
  }

  pub fn run(&self) {
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let client = Client::new(&core.handle()).unwrap();

    let url = &self.db();

    let work = client.get(url.parse().unwrap()).and_then(|res| {
        println!("Response: {}", res.status());
        println!("Headers: \n{}", res.headers());

        res.body().for_each(|chunk| {
            io::stdout().write_all(&chunk).map_err(From::from)
        })
    }).map(|_| {
        println!("\n\nDone.");
    });

    core.run(work).unwrap();
  }
}
