//! This is documentation for the `changes-stream` crate.
//!
//! The `changes-stream` crate is designed to give you a readable stream of 
//! chunked data, upon which you can register multiple handlers, that are
//! called on Read of the data chunk.

#![feature(proc_macro)]

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;
extern crate futures;
extern crate hyper;
extern crate tokio_core;

use futures::Future;
use futures::stream::Stream;

use hyper::Client;

mod package;

use package::Package;

/// A structure to generate a readable stream on which you can register handlers.
///
/// Internally, the `ChangesStream` struct holds 3 members:
///
/// | Member      | Type                                  | Notes                                                                   |
/// |-------------|---------------------------------------|-------------------------------------------------------------------------|
/// | `db`        | `String`                              | A url pointing to the data you'd like to stream.                        |
/// | `lp`        | [`tokio_core::reactor::Core`]         | The event loop                                                          |
/// | `handlers`  | `Vec<F> where F: Fn(&`[`hyper::Chunk`]`)` | A vector of handlers to be called on each Chunk from the Stream on Read |
///
/// [`tokio_core::reactor::Core`]: ../tokio_core/reactor/struct.Core.html
/// [`hyper::Chunk`]: ../hyper/struct.Chunk.html
pub struct ChangesStream {
    db: hyper::Url,
    lp: tokio_core::reactor::Core,
    handlers: Vec<Box<Fn(&Package)>>,
}

impl ChangesStream {

    /// Constructs a new `ChangesStream` struct
    ///
    /// Takes a single argument, `db`, which represents the
    /// url of the data you wish to stream.
    ///
    /// Every `ChangesStream` struct is initialized with
    /// an event loop ([`tokio_core::reactor::Core`]) and an
    /// empty vector of handlers. See above for more details.
    ///
    /// [`tokio_core::reactor::Core`]: ../tokio_core/reactor/struct.Core.html
    ///
    /// For example, to create a new `ChangesStream` struct
    /// for the npmjs registry, you would write:
    ///
    /// ```no_run
    /// # extern crate serde_json;
    /// # extern crate changes_stream;
    /// # extern crate futures;
    /// #
    /// #
    /// # use changes_stream::ChangesStream;
    /// #
    /// # fn main() {
    ///     let url = "https://replicate.npmjs.com/_changes".to_string();
    ///     let mut changes = ChangesStream::new(url);
    /// #
    /// #   changes.on(|change| {
    /// #       let data = serde_json::to_string(change).unwrap();
    /// #       println!("{}", data);
    /// #   });
    /// #
    /// #   changes.run();
    /// # }
    /// ```
    pub fn new(db: String) -> ChangesStream {
        ChangesStream {
            db: db.parse().unwrap(),
            lp: tokio_core::reactor::Core::new().unwrap(),
            handlers: vec![],
        }
    }

    /// Registers a handler. A handler is simply a function
    /// you'd like to call on a chunk from the stream at the
    /// time the chunk is read.
    ///
    /// `.on()` takes a single argument, a closure. The
    /// closure you pass should take a single [`hyper::Chunk`]
    /// as an argument.
    ///
    /// [`hyper::Chunk`]: ../hyper/struct.Chunk.html
    ///
    /// For example, to write the data in a chunk to standard
    /// out, you would write:
    ///
    /// ```no_run
    /// # extern crate serde_json;
    /// # extern crate changes_stream;
    /// # extern crate futures;
    /// #
    /// # use changes_stream::ChangesStream;
    /// #
    /// # fn main() {
    /// #   let url = "https://replicate.npmjs.com/_changes".to_string();
    /// #   let mut changes = ChangesStream::new(url);
    /// #
    ///    changes.on(|change| {
    ///       let data = serde_json::to_string(change).unwrap();
    ///       println!("{}", data);
    ///    });
    /// #
    /// #   changes.run();
    /// # }
    /// ```
    pub fn on<F: Fn(&Package) + 'static>(&mut self, handler: F) {
        self.handlers.push(Box::new(handler));
    }

    /// Runs the `ChangesStream` struct's event loop, `lp`.
    ///
    /// Call this after you have regsitered all handlers using
    /// `on`.
    ///
    /// Takes no arguments.
    ///
    /// For example:
    ///
    /// ```no_run
    /// # extern crate serde_json;
    /// # extern crate changes_stream;
    /// # extern crate futures;
    /// #
    /// # use changes_stream::ChangesStream;
    /// #
    /// # fn main() {
    /// #   let url = "https://replicate.npmjs.com/_changes".to_string();
    /// #   let mut changes = ChangesStream::new(url);
    /// #
    /// #   changes.on(|change| {
    /// #       let data = serde_json::to_string(change).unwrap();
    /// #       println!("{}", data);
    /// #   });
    /// #
    ///    changes.run();
    /// # }
    /// ```
    pub fn run(mut self) {
        let mut client = Client::new(&self.lp.handle()).unwrap();

        let handlers = self.handlers;
        self.lp
            .run(client.get(self.db).and_then(move |res| {
                assert!(res.status().is_success());

                res.body().for_each(move |chunk| {
                    for handler in &handlers {
                        let event: Package = serde_json::from_slice(&chunk).unwrap();
                        handler(&event);
                    }
                    Ok(())
                })
            }))
            .unwrap();

    }
}
