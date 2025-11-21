///
/// Module which ingests CSV data
///
use std::fs::File;
use std::sync::mpsc::Sender;
use csv::ReaderBuilder;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer};
use crate::error::ApplicationError;


#[derive(Debug, Clone, Deserialize)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct CsvTransaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    #[serde(deserialize_with = "deserialize_decimal_opt")]
    pub amount: Option<Decimal>,
}

// Use custom deserializer and actually do the rounding to 4th decimal place
fn deserialize_decimal_opt<'de, D>(deserializer: D) -> Result<Option<Decimal>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) => {
            let d = Decimal::from_str_exact(&s)
                .map_err(serde::de::Error::custom)?
                .round_dp(4); // round here
            Ok(Some(d))
        }
        None => Ok(None),
    }
}

/// Read CSV in a streaming fashion and return deserialized batch
pub fn read_csv(csv_path: &str, dispatcher_sender: Sender<CsvTransaction>) -> Result<Vec<CsvTransaction>, ApplicationError> {
    let file = File::open(csv_path)
        .map_err(|e| ApplicationError::CouldNotOpenFile(format!("{}: {}", csv_path, e)))?;

    let mut csv_reader = ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(file);


    // this is not necessary, using for unit testing
    let mut transactions = Vec::new();

    for result in csv_reader.deserialize() {
        let csv_transaction: CsvTransaction = result
            .map_err(|e| ApplicationError::FailedDeserializedCsvTransaction(format!("{}: {}", csv_path, e)))?;

        transactions.push(csv_transaction.clone());
        dispatcher_sender
            .send(csv_transaction)
            .map_err(|e| ApplicationError::Other(format!("Dispatcher channel closed: {}", e)))?;
    }

    Ok(transactions)
}


#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;
    use super::*;

    #[test]
    fn test_read_csv_real_file() {

        let path = "tests/transactions.csv"; // your real CSV file path
        let (dispatcher_sender, _dispatcher_receiver) = channel::<CsvTransaction>();

        let transactions = read_csv(path, dispatcher_sender).expect("Failed to read CSV");

        // Check that we actually read transactions
        assert!(!transactions.is_empty(), "CSV should have at least one transaction");

        // Validate each transaction
        for (row_number, tx) in transactions.iter().enumerate() {

            // Transaction type should be one of the enum variants
            match tx.tx_type {
                TransactionType::Deposit |
                TransactionType::Withdrawal |
                TransactionType::Dispute |
                TransactionType::Resolve |
                TransactionType::Chargeback => {}
            }

            // Amount rules
            match tx.tx_type {
                TransactionType::Deposit | TransactionType::Withdrawal => {
                    let amount = tx.amount.expect(&format!("Withdrawl or Deposit type {} should have amount", row_number));


                    let amount_rounded = amount.round_dp(4);
                    println!("amount {} and amount_rounded {} are:",tx.amount.unwrap(), amount_rounded);
                    assert_eq!(amount, amount_rounded, "Withdraw or Deposit type at Row {} amount not rounded to 4 decimals", row_number);
                }
                TransactionType::Dispute |
                TransactionType::Resolve |
                TransactionType::Chargeback => {
                    assert!(
                        tx.amount.is_none(),
                        "Transaction type {:?} at Row {} should not have an amount",
                        tx.tx_type,
                        row_number
                    );
                }
            }
        }
    }


    #[test]
    fn test_read_csv_malformed_file() {
        let path = "tests/malformed.csv"; // a deliberately bad CSV
        let (dispatcher_sender, _dispatcher_receiver) = channel::<CsvTransaction>();

        let err = read_csv(path, dispatcher_sender).unwrap_err();

        match err {
            ApplicationError::FailedDeserializedCsvTransaction(_) => (),
            _ => panic!("Expected FailedDeserializedCsvTransaction error"),
        }
    }
}