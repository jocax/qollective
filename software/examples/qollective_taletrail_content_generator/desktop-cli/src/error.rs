use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("NATS connection error: {0}")]
    NatsConnection(String),

    #[error("NATS request error: {0}")]
    NatsRequest(String),

    #[error("File system error: {0}")]
    FileSystem(String),

    #[error("JSON parsing error: {0}")]
    JsonParse(String),

    #[error("Template loading error: {0}")]
    TemplateLoad(String),

    #[error("Trail loading error: {0}")]
    TrailLoad(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDeserialization(#[from] toml::de::Error),

    #[error("Async NATS error: {0}")]
    AsyncNats(#[from] async_nats::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
