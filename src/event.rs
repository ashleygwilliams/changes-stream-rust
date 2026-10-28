use serde::{Deserialize, Serialize};
use serde_json::Value;

/// One event of the changes feed
#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    /// A document changed
    Change(ChangeEvent),
    /// The feed is complete
    Finished(FinishedEvent),
}

/// A changed document
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangeEvent {
    /// Update sequence of the change; a number on CouchDB 1.x, a string since 2.0
    pub seq: Value,
    /// Id of the changed document
    pub id: String,
    /// Changed revisions of the document
    pub changes: Vec<Change>,

    /// Whether the document was deleted
    #[serde(default)]
    pub deleted: bool,

    /// The document itself, present when the feed was requested with `include_docs=true`
    #[cfg(feature = "raw_value_doc")]
    #[serde(default)]
    pub doc: Option<Box<serde_json::value::RawValue>>,
    /// The document itself, present when the feed was requested with `include_docs=true`
    #[cfg(not(feature = "raw_value_doc"))]
    #[serde(default)]
    pub doc: Option<serde_json::Map<String, Value>>,
}

/// The last event of a feed
#[derive(Serialize, Deserialize, Debug)]
pub struct FinishedEvent {
    /// Update sequence to continue the feed from with `since`
    pub last_seq: Value,
    /// Number of changes not yet sent; not available on CouchDB 1.0
    pub pending: Option<u64>,
}

/// A changed revision
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Change {
    /// Revision id
    pub rev: String,
}
