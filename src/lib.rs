extern crate futures;
extern crate hyper;
extern crate tokio_core;

use futures::Future;
use futures::stream::Stream;

use hyper::Client;
use hyper::client::Response;

use std::io::{self, Write};

type Event = hyper::Chunk;

pub struct ChangesStream {
  db: hyper::Url,
  lp: tokio_core::reactor::Core,
  handlers: Option<Vec<Box<Fn(&Event)>>>,
}

impl ChangesStream {
  pub fn new(db: String) -> ChangesStream {
    ChangesStream {
      db: db.parse().unwrap(),
      lp: tokio_core::reactor::Core::new().unwrap(),
      handlers: Some(vec![]),
    }
  }

  pub fn on<F: Fn(&Event) + 'static>(&mut self, handler: F) {
    self.handlers.as_mut().unwrap().push(Box::new(handler));
  }

  pub fn run(mut self)  {
    let client = Client::new(&self.lp.handle()).unwrap();

    let handlers = self.handlers.take().unwrap();
        self.lp.run(client.get(self.db).and_then(move |res| {
            assert!(res.status().is_success());
            
            res.body().for_each(move |chunk| {
                for handler in &handlers {
                    // let event = serde_json::decode(chunk);
                    handler(&chunk);
                }
                Ok(())
            })
        })).unwrap();

  }
}
