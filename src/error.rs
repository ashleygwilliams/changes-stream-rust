#[derive(Debug)]
pub enum Error {
    RequestFailed(reqwest::Error),
    InvalidStatus(reqwest::StatusCode),
    ParsingFailed(serde_json::Error, String),
}
