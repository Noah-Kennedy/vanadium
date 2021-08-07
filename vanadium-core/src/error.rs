use thiserror::Error;

pub type VanadiumResult<T> = Result<T, VanadiumError>;

#[derive(Error, Debug)]
pub enum VanadiumError {
    #[error("File {0} not found")]
    FileNotFound(String),
    #[error("Failed to read or write data.")]
    IoError,
    #[error("Failed to parse header file")]
    InvalidHeader,
    #[error("Invalid CLI args: {0}")]
    InvalidArgs(String),
    #[error("Unknown error")]
    Unknown,
}