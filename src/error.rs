use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum YtdError {
    Api { status: u16, detail: String },
    Http(String),
    NotLoggedIn,
    Input(String),
    Json(serde_json::Error),
    Io(std::io::Error),
}

impl fmt::Display for YtdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api { status, detail } => write!(f, "Error {status}: {detail}"),
            Self::Http(msg) => write!(f, "HTTP request failed: {msg}"),
            Self::NotLoggedIn => write!(f, "Not logged in. Run `ytd login`."),
            Self::Input(msg) => write!(f, "{msg}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::Io(e) => write!(f, "{e}"),
        }
    }
}

impl std::error::Error for YtdError {}

impl From<serde_json::Error> for YtdError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<std::io::Error> for YtdError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}
