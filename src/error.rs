use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("I/O error while accessing file: {0}")]
    CouldNotOpenFile(String),  // automatically wraps std::io::Error

    #[error("CSV deserialization error: {0}")]
    FailedDeserializedCsvTransaction(String), // wraps csv::Error

    #[error("Other error: {0}")]
    Other(String),
}