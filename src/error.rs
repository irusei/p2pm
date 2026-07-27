use thiserror::Error;

pub type Result<T> = color_eyre::Result<T>;

#[derive(Error, Debug)]
pub enum P2PMError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Package not found: {0}")]
    NotFound(String),

    #[error("Repo not found: {0}")]
    RepoNotFound(String),

    #[error("No downloads for {0}")]
    NoDownloadsFound(String),

    #[error("Permission denied: {0}")]
    Permission(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Game root not set. Run 'p2pm settings' to configure your Payday 2 folder.")]
    GameRootNotSet,

    #[error("Invalid deeplink URL: {0}")]
    InvalidDeeplink(String),

    #[error("Unknown mod type: {0}")]
    UnknownType(String),
}
