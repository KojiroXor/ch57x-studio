use ch57x_domain::DomainError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PersistenceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Domain validation error: {0}")]
    Domain(#[from] DomainError),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Invalid deployment journal state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        from: crate::journal::DeploymentState,
        to: crate::journal::DeploymentState,
    },

    #[error("Journal error: {0}")]
    Journal(String),

    #[error("Corrupted data at {path}: {message}")]
    CorruptedData { path: PathBuf, message: String },

    #[error("Executable at {0} is untrusted or does not match verified hash")]
    UntrustedExecutable(PathBuf),
}
