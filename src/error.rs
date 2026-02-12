use thiserror::Error;

pub type X10Result<T> = Result<T, X10Error>;

#[derive(Debug, Error)]
pub enum X10Error {
    #[error("API key is not set")]
    ApiKeyNotSet,

    #[error("Stark account is not set")]
    StarkAccountNotSet,

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("API error (HTTP {status}): {body}")]
    Api { status: u16, body: String },

    #[error("Response error (code {code}): {message}")]
    Response { code: i64, message: String },

    #[error("Subaccount already exists")]
    SubAccountExists,

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Crypto error: {0}")]
    Crypto(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("{0}")]
    Other(String),
}
