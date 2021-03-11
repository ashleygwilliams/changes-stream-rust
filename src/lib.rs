//! The `changes-stream` crate is designed to give you a
//! futures::Stream of CouchDB changes stream events.

use bytes::Bytes;
use futures_util::stream::Stream;
use log::error;
use std::{pin::Pin, task::Poll};

mod error;
mod event;
pub use error::Error;
pub use event::{Change, ChangeEvent, Event, FinishedEvent};

/// A structure which implements futures::Stream
pub struct ChangesStream {
    /// for incomplete line chunks
    buffer: Vec<u8>,
    /// Source of http chunks provided by reqwest
    source: Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send>>,
}

impl ChangesStream {
    /// Constructs a new `ChangesStream` struct
    ///
    /// Takes a single argument, `db`, which represents the
    /// url of the data you wish to stream.
    ///
    /// For example, to create a new `ChangesStream` struct
    /// for the npmjs registry, you would write:
    ///
    /// ```no_run
    /// # use changes_stream::{ChangesStream, Event};
    /// # use futures_util::stream::StreamExt;
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// #     let url = "https://replicate.npmjs.com/_changes".to_string();
    /// #     let mut changes = ChangesStream::new(url).await.unwrap();
    /// #     while let Some(event) = changes.next().await {
    /// #         match event {
    /// #             Ok(Event::Change(change)) => println!("Change ({}): {}", change.seq, change.id),
    /// #             Ok(Event::Finished(finished)) => println!("Finished: {}", finished.last_seq),
    /// #             Err(err) => println!("Error: {:?}", err),
    /// #         }
    /// #     }
    /// # }
    /// ```
    pub async fn new(db: String) -> Result<ChangesStream, Error> {
        let res = reqwest::get(&db).await.map_err(Error::RequestFailed)?;
        let status = res.status();
        if !status.is_success() {
            return Err(Error::InvalidStatus(status));
        }
        let source = Pin::new(Box::new(res.bytes_stream()));

        Ok(ChangesStream {
            buffer: vec![],
            source,
        })
    }
}

impl Stream for ChangesStream {
    type Item = Result<Event, Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        loop {
            let line_break_pos = self
                .buffer
                .iter()
                .enumerate()
                .find(|(_pos, b)| **b == 0x0A) // search for \n
                .map(|(pos, _b)| pos);
            if let Some(line_break_pos) = line_break_pos {
                let mut line = self.buffer.drain(0..=line_break_pos).collect::<Vec<_>>();

                if line.len() < 15 {
                    // skip prologue, epilogue and empty lines (continuous mode)
                    continue;
                }
                line.remove(line.len() - 1); // remove \n
                if line[line.len() - 1] == 0x0D {
                    // 0x0D is '\r'. CouchDB >= 2.0 sends "\r\n"
                    line.remove(line.len() - 1);
                }
                if line[line.len() - 1] == 0x2C {
                    // 0x2C is ','
                    line.remove(line.len() - 1); // remove ,
                }

                let result = match serde_json::from_slice(line.as_slice()) {
                    Err(err) => Err(Error::ParsingFailed(
                        err,
                        String::from_utf8(line).unwrap_or_default(),
                    )),
                    Ok(json) => {
                        let event: Event = json;
                        Ok(event)
                    }
                };

                return Poll::Ready(Some(result));
            } else {
                match Stream::poll_next(self.source.as_mut(), cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Ready(Some(Ok(chunk))) => self.buffer.append(&mut chunk.to_vec()),
                    Poll::Ready(Some(Err(err))) => {
                        error!("Error getting next chunk: {:?}", err);
                        return Poll::Ready(None);
                    }
                };
            }
        }
    }
}
