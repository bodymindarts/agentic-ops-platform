use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("ConfigError - ReadFile: {0}")]
    ReadFile(#[from] std::io::Error),
    #[error("ConfigError - ParseYaml: {0}")]
    ParseYaml(#[from] serde_yaml::Error),
    #[error("ConfigError - Validation: {0}")]
    Validation(String),
}

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("CoreError - Config: {0}")]
    Config(#[from] ConfigError),
}
