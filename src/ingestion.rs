use std::fs::File;
use serde::Deserialize;
use crate::error::ApplicationError;


#[derive(Debug, Deserialize)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub struct CsvTransaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    pub tx: u32,
    pub amount: Option<String>, // keep as String for now
}

/// Read CSV in a streaming fashion and return deserialized batch
pub fn read_csv(path: &str) -> Result<Vec<CsvTransaction>, ApplicationError> {
    let file = File::open(path)
        .map_err(|e| ApplicationError::CouldNotOpenFile(format!("{}: {}", path, e)))?;

    let mut rdr = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(file);

    let mut transactions = Vec::new();

    for result in rdr.deserialize() {
        let tx: CsvTransaction = result
            .map_err(|e| ApplicationError::FailedDeserializedCsvTransaction(format!("{}: {}", path, e)))?;

        // dummy update account for now
        dummy_update_account(&tx);
        transactions.push(tx);
    }

    Ok(transactions)
}

/// Temporary placeholder to mimic processing
fn dummy_update_account(tx: &CsvTransaction) {
    println!("Processing transaction: {:?}", tx);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_csv_real_file() {
        let path = "tests/transactions.csv"; // your real CSV file path
        let transactions = read_csv(path).expect("Failed to read CSV");

        assert!(!transactions.is_empty(), "CSV should have at least one transaction");

        // example checks on the first row
        let first_tx = &transactions[0];
        assert!(matches!(first_tx.tx_type, TransactionType::Deposit | TransactionType::Withdrawal));
        assert!(first_tx.client_id > 0);
        assert!(first_tx.tx > 0);
        assert!(first_tx.amount.is_some());
    }

    #[test]
    fn test_read_csv_malformed_file() {
        let path = "tests/malformed.csv"; // a deliberately bad CSV
        let err = read_csv(path).unwrap_err();

        match err {
            ApplicationError::FailedDeserializedCsvTransaction(_) => (),
            _ => panic!("Expected FailedDeserializedCsvTransaction error"),
        }
    }
}