use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("CoreError - Config: {0}")]
    Config(String),
}
