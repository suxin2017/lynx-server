use thiserror::Error;

/// Specific error types for request processing operations
#[derive(Error, Debug)]
pub enum RequestProcessingError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Rule not found with ID: {id}")]
    RuleNotFound { id: i32 },

    #[error("Invalid capture pattern: {pattern}, reason: {reason}")]
    InvalidCapturePattern { pattern: String, reason: String },

    #[error("Invalid handler configuration for type {handler_type}: {reason}")]
    InvalidHandlerConfig {
        handler_type: String,
        reason: String,
    },

    #[error("Rule validation failed: {reason}")]
    RuleValidation { reason: String },

    #[error("Transaction failed: {reason}")]
    Transaction { reason: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Pattern compilation error: {0}")]
    PatternCompilation(#[from] glob::PatternError),

    #[error("Regex compilation error: {0}")]
    RegexCompilation(#[from] regex::Error),
}

pub type Result<T> = std::result::Result<T, RequestProcessingError>;
