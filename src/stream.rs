use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use bytes::{Buf, Bytes};
use futures_util::stream::Stream;
use serde_json::Value;
use url::Url;

use crate::{ChangeEvent, Error, Event, FinishedEvent};

/// Lines shorter than this carry no event: the normal feed's prologue `{"results":[` and
/// epilogue `],`, and the empty heartbeat lines of the continuous feed.
const MIN_EVENT_LINE_LEN: usize = 14;
/// Connect and read timeout of the changes request
const TIMEOUT: Duration = Duration::from_secs(60);

type ByteSource = Pin<Box<dyn Stream<Item = reqwest::Result<Bytes>> + Send>>;

/// A [`Stream`] of the events of a CouchDB changes feed.
///
/// Every line of the response becomes one [`Event`]. A line that cannot be parsed yields
/// [`Error::ParsingFailed`] and the stream continues. A failed read (connection loss, or no data
/// for 60 seconds) yields [`Error::RequestFailed`] once, after which the stream ends. For a
/// continuous feed, request a `heartbeat` shorter than 60 seconds so an idle feed does not run
/// into the read timeout.
pub struct ChangesStream {
    /// metrics bytes counter
    #[cfg(feature = "metrics")]
    bytes: metrics::Counter,
    /// metrics entries counter
    #[cfg(feature = "metrics")]
    entries: metrics::Counter,
    /// Source of http chunks provided by reqwest
    source: ByteSource,
    /// The current line, still waiting for its '\n'
    line: Vec<u8>,
    /// Part of the last chunk that has not been split into lines yet
    chunk: Bytes,
    /// Set once the source has ended or failed; from then on the stream only returns `None`
    done: bool,
}

impl ChangesStream {
    /// Constructs a new `ChangesStream` struct with a post payload
    ///
    /// Takes two arguments, `db`, which represents the
    /// url of the data you wish to stream, and `payload`, the json body of the request.
    ///
    /// For example, to create a new `ChangesStream` struct
    /// for the npmjs registry, you would write:
    ///
    /// ```no_run
    /// # use changes_stream2::{ChangesStream, Event};
    /// # use futures_util::stream::StreamExt;
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// #     let url = "https://replicate.npmjs.com/_changes?filter=_selector".to_string();
    /// #     let mut changes = ChangesStream::with_post_payload(url, &serde_json::json!({"selector": { "_id": { "$regex": "^_design/" }}})).await.unwrap();
    /// #     while let Some(event) = changes.next().await {
    /// #         match event {
    /// #             Ok(Event::Change(change)) => println!("Change ({}): {}", change.seq, change.id),
    /// #             Ok(Event::Finished(finished)) => println!("Finished: {}", finished.last_seq),
    /// #             Err(err) => println!("Error: {:?}", err),
    /// #         }
    /// #     }
    /// # }
    /// ```
    ///
    /// Fails if `db` is not a valid url, the request fails or the server answers with a
    /// non-success status.
    pub async fn with_post_payload(db: String, payload: &Value) -> Result<Self, Error> {
        let url = Url::parse(&db)?;
        let client = reqwest::ClientBuilder::new()
            .connect_timeout(TIMEOUT)
            .read_timeout(TIMEOUT)
            .build()?;
        let res = client.post(url.clone()).json(payload).send().await?;
        let status = res.status();
        if !status.is_success() {
            let body = res.text().await.unwrap_or_default();
            return Err(Error::InvalidResponse { status, body });
        }

        Ok(Self::from_source(Box::pin(res.bytes_stream()), &url))
    }

    /// Constructs a new `ChangesStream` struct
    ///
    /// Takes a single argument, `db`, which represents the
    /// url of the data you wish to stream.
    ///
    /// For example, to create a new `ChangesStream` struct
    /// for the npmjs registry, you would write:
    ///
    /// ```no_run
    /// # use changes_stream2::{ChangesStream, Event};
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
    ///
    /// Fails like [`ChangesStream::with_post_payload`].
    pub async fn new(db: String) -> Result<Self, Error> {
        Self::with_post_payload(db, &serde_json::json!({})).await
    }

    /// Splits the bytes of `source` into events. `url` only names the metrics.
    #[cfg_attr(not(feature = "metrics"), allow(unused_variables))]
    pub(crate) fn from_source(source: ByteSource, url: &Url) -> Self {
        #[cfg(feature = "metrics")]
        let (bytes, entries) = {
            let database = database_label(url);
            let bytes_name = "couchdb_changes_bytes_total";
            let entries_name = "couchdb_changes_entries_total";
            metrics::describe_counter!(bytes_name, metrics::Unit::Bytes, "Changes stream bytes");
            metrics::describe_counter!(
                entries_name,
                metrics::Unit::Count,
                "Changes stream entries"
            );
            (
                metrics::counter!(bytes_name, "database" => database.clone()),
                metrics::counter!(entries_name, "database" => database),
            )
        };

        Self {
            #[cfg(feature = "metrics")]
            bytes,
            #[cfg(feature = "metrics")]
            entries,
            source,
            line: Vec::new(),
            chunk: Bytes::new(),
            done: false,
        }
    }

    /// Parses the buffered line and empties the buffer, keeping its allocation.
    /// Returns `None` for lines that carry no event.
    fn take_line(&mut self) -> Option<Result<Event, Error>> {
        let result = parse_line(&self.line);
        self.line.clear();

        #[cfg(feature = "metrics")]
        if matches!(result, Some(Ok(_))) {
            self.entries.increment(1);
        }

        result
    }
}

impl Stream for ChangesStream {
    type Item = Result<Event, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;
        loop {
            if this.done {
                return Poll::Ready(None);
            }

            if let Some(end) = this.chunk.iter().position(|&byte| byte == b'\n') {
                this.line.extend_from_slice(&this.chunk[..end]);
                this.chunk.advance(end + 1);
                match this.take_line() {
                    Some(result) => return Poll::Ready(Some(result)),
                    None => continue,
                }
            }

            // The line continues in the next chunk
            this.line.extend_from_slice(&this.chunk);
            this.chunk.clear();

            match this.source.as_mut().poll_next(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Some(Ok(chunk))) => {
                    #[cfg(feature = "metrics")]
                    this.bytes.increment(chunk.len() as u64);

                    this.chunk = chunk;
                }
                Poll::Ready(Some(Err(err))) => {
                    this.done = true;
                    return Poll::Ready(Some(Err(Error::RequestFailed(err))));
                }
                Poll::Ready(None) => {
                    // The body may end without a final '\n'
                    this.done = true;
                    return Poll::Ready(this.take_line());
                }
            }
        }
    }
}

/// Parses one line of the feed, without its '\n'. Returns `None` for lines that carry no event.
fn parse_line(line: &[u8]) -> Option<Result<Event, Error>> {
    // CouchDB >= 2.0 ends lines with "\r\n"
    let line = line.strip_suffix(b"\r").unwrap_or(line);
    // The results of the normal feed are separated by ','
    let line = line.strip_suffix(b",").unwrap_or(line);
    if line.len() < MIN_EVENT_LINE_LEN {
        return None;
    }

    if line.starts_with(br#""last_seq""#) {
        // The last line of the normal feed closes the object opened by `{"results":[`
        let mut object = Vec::with_capacity(line.len() + 1);
        object.push(b'{');
        object.extend_from_slice(line);
        let result = serde_json::from_slice::<FinishedEvent>(&object)
            .map(Event::Finished)
            .map_err(|error| parsing_failed(error, line));
        return Some(result);
    }

    let result = serde_json::from_slice::<ChangeEvent>(line)
        .map(Event::Change)
        .or_else(|error| {
            serde_json::from_slice::<FinishedEvent>(line)
                .map(Event::Finished)
                .map_err(|_err| parsing_failed(error, line))
        });
    Some(result)
}

fn parsing_failed(error: serde_json::Error, line: &[u8]) -> Error {
    Error::ParsingFailed {
        error,
        json: String::from_utf8_lossy(line).into_owned(),
    }
}

/// Host and path of `url` with every run of '_' and '/' collapsed to a single '_'
#[cfg(feature = "metrics")]
fn database_label(url: &Url) -> String {
    let raw = format!("{}_{}", url.host_str().unwrap_or_default(), url.path());
    let mut label = String::with_capacity(raw.len());
    for c in raw.chars() {
        match c {
            '_' | '/' if label.ends_with('_') => {}
            '_' | '/' => label.push('_'),
            c => label.push(c),
        }
    }
    label
}

#[cfg(test)]
mod tests {
    use futures_util::stream::{self, StreamExt};

    use super::*;

    fn stream_of(chunks: &[&'static [u8]]) -> ChangesStream {
        let chunks: Vec<reqwest::Result<Bytes>> = chunks
            .iter()
            .map(|chunk| Ok(Bytes::from_static(chunk)))
            .collect();
        let url = Url::parse("http://localhost:5984/db/_changes").expect("valid test url");
        ChangesStream::from_source(Box::pin(stream::iter(chunks)), &url)
    }

    async fn events_of(chunks: &[&'static [u8]]) -> Vec<Result<Event, Error>> {
        stream_of(chunks).collect().await
    }

    fn change_ids(events: &[Result<Event, Error>]) -> Vec<&str> {
        events
            .iter()
            .filter_map(|event| match event {
                Ok(Event::Change(change)) => Some(change.id.as_str()),
                _ => None,
            })
            .collect()
    }

    #[tokio::test]
    async fn line_split_across_chunks_yields_one_change() {
        let events = events_of(&[
            br#"{"seq":1,"id":"a","cha"#,
            br#"nges":[{"rev":"1-x"}]}"#,
            b"\n",
        ])
        .await;
        assert_eq!(change_ids(&events), ["a"]);
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn crlf_and_trailing_comma_are_stripped() {
        let events = events_of(&[
            b"{\"seq\":1,\"id\":\"a\",\"changes\":[{\"rev\":\"1-x\"}]},\r\n",
            b"{\"seq\":2,\"id\":\"b\",\"changes\":[{\"rev\":\"1-y\"}]}\r\n",
        ])
        .await;
        assert_eq!(change_ids(&events), ["a", "b"]);
    }

    #[tokio::test]
    async fn normal_feed_yields_changes_then_finished() {
        let events = events_of(&[b"{\"results\":[\r\n\
            {\"seq\":\"1-abc\",\"id\":\"a\",\"changes\":[{\"rev\":\"1-x\"}]},\r\n\
            {\"seq\":\"2-abc\",\"id\":\"b\",\"changes\":[{\"rev\":\"1-y\"}],\"deleted\":true}\r\n\
            ],\r\n\
            \"last_seq\":\"2-abc\",\"pending\":0}\r\n"])
        .await;
        assert_eq!(change_ids(&events), ["a", "b"]);
        let Some(Ok(Event::Finished(finished))) = events.last() else {
            panic!("expected the normal feed to end with Finished, got {events:?}");
        };
        assert_eq!(finished.last_seq, "2-abc");
        assert_eq!(finished.pending, Some(0));
    }

    #[tokio::test]
    async fn continuous_feed_skips_heartbeats_and_yields_finished() {
        let events = events_of(&[
            b"{\"seq\":1,\"id\":\"a\",\"changes\":[{\"rev\":\"1-x\"}]}\n",
            b"\n\n",
            b"{\"last_seq\":1,\"pending\":0}\n",
        ])
        .await;
        assert_eq!(change_ids(&events), ["a"]);
        assert!(matches!(events.last(), Some(Ok(Event::Finished(_)))));
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn last_line_without_newline_is_emitted() {
        let events = events_of(&[br#"{"seq":1,"id":"a","changes":[{"rev":"1-x"}]}"#]).await;
        assert_eq!(change_ids(&events), ["a"]);
    }

    #[tokio::test]
    async fn invalid_json_yields_parsing_failed_and_continues() {
        let events = events_of(&[
            b"{\"this is\": not json}\n",
            b"{\"seq\":1,\"id\":\"a\",\"changes\":[{\"rev\":\"1-x\"}]}\n",
        ])
        .await;
        let Some(Err(Error::ParsingFailed { json, .. })) = events.first() else {
            panic!("expected ParsingFailed first, got {events:?}");
        };
        assert_eq!(json, "{\"this is\": not json}");
        assert_eq!(change_ids(&events), ["a"]);
    }

    #[tokio::test]
    async fn stream_stays_finished_after_end() {
        let mut stream = stream_of(&[]);
        assert!(stream.next().await.is_none());
        assert!(stream.next().await.is_none());
    }

    #[cfg(feature = "metrics")]
    #[test]
    fn database_label_collapses_underscores_and_slashes() {
        let url = Url::parse("https://couch.example.com/my_db/_changes").expect("valid test url");
        assert_eq!(database_label(&url), "couch.example.com_my_db_changes");
    }
}
