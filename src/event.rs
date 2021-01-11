use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Event {
    Change(ChangeEvent),
    Finished(FinishedEvent),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangeEvent {
    pub seq: Value,
    pub id: String,
    pub changes: Vec<Change>,

    #[serde(default)]
    pub deleted: bool,

    #[serde(default)]
    pub doc: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FinishedEvent {
    pub last_seq: Value,
    pub pending: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Change {
    pub rev: String,
}
