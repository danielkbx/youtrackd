use std::fmt;

#[derive(Debug)]
#[allow(dead_code)]
pub enum YtdError {
    Api { status: u16, detail: String },
    Http(String),
    PermissionDenied(String),
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
            Self::PermissionDenied(msg) => write!(f, "Permission denied: {msg}"),
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

impl YtdError {
    pub fn from_api(status: u16, detail: impl Into<String>) -> Self {
        let detail = detail.into();

        if status == 403 || looks_like_permission_error(&detail) {
            return Self::PermissionDenied(detail);
        }

        Self::Api { status, detail }
    }
}

fn looks_like_permission_error(detail: &str) -> bool {
    let normalized = detail.to_ascii_lowercase();
    normalized.contains("forbidden")
        || normalized.contains("permission")
        || normalized.contains("not allowed")
        || normalized.contains("access denied")
}

#[cfg(test)]
mod tests {
    use super::YtdError;

    #[test]
    fn classifies_status_403_as_permission_denied() {
        let err = YtdError::from_api(403, "Forbidden");
        assert!(matches!(err, YtdError::PermissionDenied(_)));
    }

    #[test]
    fn classifies_permission_text_as_permission_denied() {
        let err = YtdError::from_api(400, "User has no permission to do this");
        assert!(matches!(err, YtdError::PermissionDenied(_)));
    }
}
