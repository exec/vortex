use thiserror::Error;

pub type Result<T> = std::result::Result<T, VortexError>;

#[derive(Error, Debug)]
pub enum VortexError {
    #[error("VM operation failed: {message}")]
    VmError { message: String },

    #[error("Backend not available: {backend}")]
    BackendUnavailable { backend: String },

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Network error: {message}")]
    NetworkError { message: String },

    #[error("Storage error: {message}")]
    StorageError { message: String },

    #[error("Authentication error: {message}")]
    AuthError { message: String },

    #[error("Plugin error: {message}")]
    PluginError { message: String },

    #[error("Resource limit exceeded: {resource}")]
    ResourceLimitExceeded { resource: String },

    #[error("Permission denied: {action}")]
    PermissionDenied { action: String },

    #[error("Invalid input: {field} - {message}")]
    InvalidInput { field: String, message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}
