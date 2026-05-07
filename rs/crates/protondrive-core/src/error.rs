use thiserror::Error;

pub type Result<T, E = DriveError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum DriveError {
    #[error("authentication error: {0}")]
    Auth(String),

    #[error("API error {code}: {message}")]
    Api { code: u32, message: String },

    #[error("network error: {0}")]
    Network(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("not found")]
    NotFound,

    #[error("permission denied")]
    PermissionDenied,

    #[error("rate limited; retry after {retry_after_secs}s")]
    RateLimited { retry_after_secs: u64 },

    #[error("event stream gap; re-sync required from event {latest_event_id}")]
    EventStreamGap { latest_event_id: String },

    #[error("checksum mismatch on block {block_index}")]
    ChecksumMismatch { block_index: usize },

    #[error("invalid app version string")]
    InvalidAppVersion,

    #[error("session expired")]
    SessionExpired,

    #[error("2FA required")]
    TwoFactorRequired,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
