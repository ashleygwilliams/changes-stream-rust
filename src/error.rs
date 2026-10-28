use thiserror::Error;

/// Errors of creating or reading a [`ChangesStream`](crate::ChangesStream)
#[derive(Error, Debug)]
pub enum Error {
    /// The database url could not be parsed
    #[error("Could not parse the url: {0:?}")]
    InvalidUrl(#[from] url::ParseError),
    /// The request failed or reading the response broke off; the stream ends after this error
    #[error("Request failed: {0:?}")]
    RequestFailed(#[from] reqwest::Error),
    /// The server answered the changes request with a non-success status
    #[error("Server answered with non-ok status: {status}. body: {body}")]
    InvalidResponse {
        status: reqwest::StatusCode,
        body: String,
    },
    /// A line of the feed is neither a change nor the final event; the stream continues
    #[error("Could not parse server response: {json}")]
    ParsingFailed {
        #[source]
        error: serde_json::Error,
        json: String,
    },
}
