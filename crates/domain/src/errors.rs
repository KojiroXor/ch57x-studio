use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum PlatformError {
    #[error("System API error: {0}")]
    SystemApi(String),

    #[error("Window not found: {0}")]
    WindowNotFound(String),

    #[error("Process launch failed: {0}")]
    ProcessLaunchFailed(String),

    #[error("Permission denied (UIPI integrity barrier or access denied)")]
    PermissionDenied,

    #[error("Feature not supported on current platform/compositor: {0}")]
    Unsupported(String),

    #[error("Hotkey registration conflict: {0}")]
    HotkeyConflict(String),

    #[error("Timeout occurred after {0} ms")]
    Timeout(u64),

    #[error("Internal platform failure: {0}")]
    Internal(String),
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("Invalid slot index: {0}")]
    InvalidSlot(usize),

    #[error("Invalid trigger bank or key: {0}")]
    InvalidTrigger(String),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}
