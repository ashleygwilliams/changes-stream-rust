use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub seq: u64,
    pub id: String,
    pub changes: Vec<Change>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Change {
    pub rev: String,
}
