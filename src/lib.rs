//! The `changes-stream` crate is designed to give you a
//! futures::Stream of CouchDB changes stream events.

#![forbid(unsafe_code)]
#![forbid(clippy::unwrap_used)]

mod error;
mod event;
mod stream;

pub use error::Error;
pub use event::{Change, ChangeEvent, Event, FinishedEvent};
pub use stream::ChangesStream;
