#[derive(Serialize, Deserialize, Debug)]
pub struct Package {
    seq: u64,
    id: String,
    changes: Vec<Change>
}

#[derive(Serialize, Deserialize, Debug)]
struct Change {
  rev: String,
}
