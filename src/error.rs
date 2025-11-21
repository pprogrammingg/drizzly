use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("I/O error while accessing file: {0}")]
    CouldNotOpenFile(String),

    #[error("CSV deserialization error: {0}")]
    FailedDeserializedCsvTransaction(String),

    #[error("Client has insufficient balance for withdrawal. More info: client-id {0}, tx-id {1}")]
    InsufficientAvailableBalanceForWithdrawal(u16, u32),

    #[error("Client account is frozen, cannot perform transaction. More info: client-id {0}, tx-id {1}")]
    ClientAccountFrozen(u16, u32),


    #[error("Other error: {0}")]
    Other(String),
}