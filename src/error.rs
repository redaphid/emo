use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum EmoError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidInput(String),
    ConfigError(String),
}

impl fmt::Display for EmoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmoError::Io(e) => write!(f, "IO error: {}", e),
            EmoError::Json(e) => write!(f, "JSON error: {}", e),
            EmoError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            EmoError::ConfigError(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl Error for EmoError {}

impl From<io::Error> for EmoError {
    fn from(error: io::Error) -> Self {
        EmoError::Io(error)
    }
}

impl From<serde_json::Error> for EmoError {
    fn from(error: serde_json::Error) -> Self {
        EmoError::Json(error)
    }
}

pub type Result<T> = std::result::Result<T, EmoError>;
