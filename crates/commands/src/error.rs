use adhd_ranch_domain::DomainError;
use adhd_ranch_storage::FocusStoreError;

#[derive(Debug, serde::Serialize)]
#[serde(tag = "type", content = "message", rename_all = "snake_case")]
pub enum CommandError {
    BadRequest(String),
    NotFound(String),
    AlreadyExists(String),
    Internal(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BadRequest(m) => write!(f, "{m}"),
            Self::NotFound(m) => write!(f, "{m}"),
            Self::AlreadyExists(m) => write!(f, "{m}"),
            Self::Internal(m) => write!(f, "{m}"),
        }
    }
}

impl std::error::Error for CommandError {}

impl From<FocusStoreError> for CommandError {
    fn from(e: FocusStoreError) -> Self {
        match e {
            FocusStoreError::NotFound(_) => CommandError::NotFound(e.to_string()),
            FocusStoreError::AlreadyExists(_) => CommandError::AlreadyExists(e.to_string()),
            FocusStoreError::TaskIndexOutOfRange { .. } => CommandError::BadRequest(e.to_string()),
            FocusStoreError::Io(_) | FocusStoreError::Parse { .. } => {
                CommandError::Internal(e.to_string())
            }
        }
    }
}

impl From<DomainError> for CommandError {
    fn from(e: DomainError) -> Self {
        CommandError::BadRequest(e.to_string())
    }
}
