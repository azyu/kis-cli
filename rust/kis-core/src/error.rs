use thiserror::Error;

#[derive(Debug, Error)]
pub enum KisError {
    #[error("config error: {0}")]
    Config(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("API error [{code}]: {message}")]
    Api { code: String, message: String },
    #[error("parse error: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, KisError>;
