use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForgeWelcomeError {
    #[error("Failed to read manifest: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse manifest: {0}")]
    Yaml(#[from] serde_yaml::Error),
}
