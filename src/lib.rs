//! The `changes-stream` crate is designed to give you a
//! futures::Stream of CouchDB changes stream events.

use bytes::Bytes;
use futures_util::stream::Stream;
use std::{pin::Pin, task::Poll};

mod event;
pub use event::Event;

/// A structure which implements futures::Stream
pub struct ChangesStream {
    /// for incomplete line chunks
    buffer: Vec<u8>,
    /// Source of http chunks provided by reqwest
    source: Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>>>>,
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
    /// #     let mut changes = ChangesStream::new(url).await;
    /// #     while let Some(event) = changes.next().await {
    /// #         match event {
    /// #             Event::Change(change) => {
    /// #                 println!("{}: {}", change.seq, change.id);
    /// #             }
    /// #             Event::Finished(finished) => {
    /// #                 println!("Finished: {}", finished.last_seq);
    /// #             }
    /// #         }
    /// #     }
    /// # }
    /// ```
    pub async fn new(db: String) -> ChangesStream {
        let res = reqwest::get(&db).await.unwrap();
        assert!(res.status().is_success());
        let source = Pin::new(Box::new(res.bytes_stream()));

        ChangesStream {
            buffer: vec![],
            source,
        }
    }
}

impl Stream for ChangesStream {
    type Item = Event;

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
                match serde_json::from_slice(line.as_slice()) {
                    Err(err) => {
                        panic!(
                            "Error: {:?} \"{}\"",
                            err,
                            std::str::from_utf8(line.as_slice()).unwrap()
                        );
                    }
                    Ok(json) => {
                        let event: Event = json;
                        return Poll::Ready(Some(event));
                    }
                }
            } else {
                match Stream::poll_next(self.source.as_mut(), cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(None) => return Poll::Ready(None),
                    Poll::Ready(Some(Ok(chunk))) => self.buffer.append(&mut chunk.to_vec()),
                    Poll::Ready(Some(Err(err))) => panic!("Error getting next chunk: {:?}", err),
                };
            }
        }
    }
}
