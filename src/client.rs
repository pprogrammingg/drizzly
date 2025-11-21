use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use rust_decimal::Decimal;
use rust_decimal::prelude::{Zero};
use crate::csv_ingestor::CsvTransaction;
use crate::error::ApplicationError;
use crate::error::ApplicationError::InsufficientAvailableBalanceForWithdrawal;

/// a thread-safe mutable hashmap which holds client-id vs state
pub type GlobalClientsMap = Arc<RwLock<HashMap<u16, Client>>>;

pub fn new_clients_map() -> GlobalClientsMap {
    Arc::new(RwLock::new(HashMap::new()))
}


/// Client holds client state include tx history
#[derive(Debug, Default)]
pub struct Client {
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    pub locked: bool,

    pub tx_history: HashMap<u32, CsvTransaction>,
}


impl Client {
     pub fn deposit(&mut self, tx: &CsvTransaction) {
        match tx.amount {
            Some(amount) => {
                self.available += amount;
                self.total += amount;
                self.tx_history.insert(tx.tx_id, tx.clone());
            }
            None => {
                // Do nothing if amount is None
            }
        }
    }

    pub fn withdraw(&mut self, tx: &CsvTransaction) -> Result<(), ApplicationError> {
        let amount = tx.amount.unwrap_or_else(Decimal::zero);

        if self.available >= amount {
            self.available -= amount;
            self.total -= amount;
            self.tx_history.insert(tx.tx_id, tx.clone());
            Ok(())
        } else {
            Err(InsufficientAvailableBalanceForWithdrawal(
                tx.client_id, tx.tx_id),
            )
        }
    }

    /// A dispute represents a client's claim that a transaction was erroneous and should be reversed.
    /// Funds should be held: available decreases, held increases, total remains the same.
    /// Ignore non-existing transactions.
    pub fn dispute(&mut self, tx_id: u32) {
        if let Some(tx) = self.tx_history.get(&tx_id) {
            //eprintln!("tx is {:#?}", tx);
            match tx.amount {
                Some(amount) => {
                    self.available -= amount;
                    self.held += amount;
                }
                None => {
                    eprintln!("WARNING: referenced tx for dispute had no amount!!!, {}:{}", tx.client_id, tx.tx_id );
                }
            }
        }else {
            eprintln!("WARNING: referenced tx for dispute does not exist in history!!!, {tx_id}" );
        }
    }

    /// A resolve represents the resolution to a dispute, releasing held funds.
    /// Held decreases, available increases, total remains the same.
    /// Ignore non-existing transactions or transactions not under dispute.
    pub fn resolve(&mut self, tx_id: u32) {
        if let Some(tx) = self.tx_history.get(&tx_id) {

            // assuming when a transaction is found it is under dispute
            match tx.amount {
                Some(amount) => {
                    self.held -= amount;
                    self.available += amount;
                }
                None => {
                    eprintln!("WARNING: referenced tx for resolve had no amount!!!, {}:{}", tx.client_id, tx.tx_id  );
                }
            }
        }else {
            eprintln!("WARNING: referenced tx for resolve does not exist in history!!!, {tx_id}" );
        }
    }

    /// A chargeback is the final state of a dispute, reversing the transaction.
    /// Held and total decrease by the disputed amount, and the client account is locked.
    /// Ignore non-existing transactions or transactions not under dispute.
    pub fn chargeback(&mut self, tx_id: u32) {
        if let Some(tx) = self.tx_history.get(&tx_id) {
            match tx.amount {
                Some(amount) => {
                    self.held -= amount;
                    self.total -= amount;
                    self.locked = true;
                }
                None => {
                    eprintln!("WARNING: referenced tx for chargeback had no amount!!!, {}:{}", tx.client_id, tx.tx_id ) ;
                }
            }
        } else {
           eprintln!("WARNING: referenced tx for chargeback does not exist in history!!!, {tx_id}" );
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::FromPrimitive;
    use crate::csv_ingestor::TransactionType;
    use crate::error::ApplicationError;

    // helper to create CsvTransaction
    fn make_tx(client_id: u16, tx_id: u32, amount: Option<f64>, tx_type: TransactionType) -> CsvTransaction {
        CsvTransaction {
            client_id,
            tx_id,
            tx_type,
            amount: amount.map(|a| Decimal::from_f64(a).unwrap()),
        }
    }

    #[test]
    fn test_deposit_and_withdraw() {
        let mut client = Client::default();

        let dep = make_tx(1, 1, Some(100.0), TransactionType::Deposit);
        client.deposit(&dep);

        assert_eq!(client.available, Decimal::from_f64(100.0).unwrap());
        assert_eq!(client.held, Decimal::zero());
        assert_eq!(client.total, Decimal::from_f64(100.0).unwrap());

        let withdrawal = make_tx(1, 2, Some(40.0), TransactionType::Withdrawal);
        client.withdraw(&withdrawal).unwrap();

        assert_eq!(client.available, Decimal::from_f64(60.0).unwrap());
        assert_eq!(client.held, Decimal::zero());
        assert_eq!(client.total, Decimal::from_f64(60.0).unwrap());

        // Withdraw more than available should error
        let bad_withdrawal = make_tx(1, 3, Some(100.0), TransactionType::Withdrawal);
        let err = client.withdraw(&bad_withdrawal).unwrap_err();
        match err {
            InsufficientAvailableBalanceForWithdrawal(client_id, tx_id) => {
                assert_eq!(client_id, 1);
                assert_eq!(tx_id, 3);
            }
            _ => panic!("Expected InsufficientAvailableBalanceForWithdrawal error"),
        }
    }

    #[test]
    fn test_dispute_resolve_chargeback() {
        let mut client = Client::default();

        let dep = make_tx(1, 1, Some(100.0), TransactionType::Deposit);
        client.deposit(&dep);

        // dispute
        client.dispute(1);
        assert_eq!(client.available, Decimal::from_f64(0.0).unwrap());
        assert_eq!(client.held, Decimal::from_f64(100.0).unwrap());
        assert_eq!(client.total, Decimal::from_f64(100.0).unwrap());
        assert!(!client.locked);

        // resolve
        client.resolve(1);
        assert_eq!(client.available, Decimal::from_f64(100.0).unwrap());
        assert_eq!(client.held, Decimal::from_f64(0.0).unwrap());
        assert_eq!(client.total, Decimal::from_f64(100.0).unwrap());
        assert!(!client.locked);

        // dispute again
        client.dispute(1);
        // chargeback
        client.chargeback(1);
        assert_eq!(client.available, Decimal::from_f64(0.0).unwrap());
        assert_eq!(client.held, Decimal::from_f64(0.0).unwrap());
        assert_eq!(client.total, Decimal::from_f64(0.0).unwrap());
        assert!(client.locked);
    }

    #[test]
    fn test_dispute_nonexistent_tx() {
        let mut client = Client::default();

        // disputing a non-existent transaction should do nothing
        client.dispute(999);
        client.resolve(999);
        client.chargeback(999);

        assert_eq!(client.available, Decimal::from_f64(0.0).unwrap());
        assert_eq!(client.held, Decimal::from_f64(0.0).unwrap());
        assert_eq!(client.total, Decimal::from_f64(0.0).unwrap());
        assert!(!client.locked);
    }

    #[test]
    fn test_deposit_without_amount() {
        let mut client = Client::default();
        let tx = make_tx(1, 10, None, TransactionType::Deposit);
        client.deposit(&tx);

        // no change since amount is None
        assert_eq!(client.available, Decimal::from_f64(0.0).unwrap());
        assert_eq!(client.total, Decimal::from_f64(0.0).unwrap());
    }
}