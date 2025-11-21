use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("I/O error while accessing file: {0}")]
    CouldNotOpenFile(String),

    #[error("CSV deserialization error: {0}")]
    FailedDeserializedCsvTransaction(String),

    #[error("Client has insufficient balance for withdrawal: {0}")]
    InsufficientAvailableBalanceForWithdrawal(String),

    #[error("Client account is frozen, cannot perform transaction. More info: {0}")]
    ClientAccountFrozen(String),


    #[error("Other error: {0}")]
    Other(String),
}