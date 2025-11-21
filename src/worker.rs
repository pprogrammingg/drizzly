use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use crate::client::{GlobalClientsMap};
use crate::csv_ingestor::{CsvTransaction, TransactionType};
use crate::error::ApplicationError;

// to make types simpler
type WorkerSender = Sender<CsvTransaction>;
type WorkerHandle = JoinHandle<Result<(), ApplicationError>>;

/// Spawn worker threads for parallel processing
/// Used as initialization method in main.rs
pub fn spawn_workers(global_clients_map: GlobalClientsMap) -> (Vec<WorkerSender>, Vec<WorkerHandle>) {
    let num_workers = num_cpus::get();
    let mut worker_senders = Vec::with_capacity(num_workers);
    let mut worker_handles = Vec::with_capacity(num_workers);

    for worker_id in 0..num_workers {
        let (sender, receiver) = std::sync::mpsc::channel();
        worker_senders.push(sender);

        let clients_ref = global_clients_map.clone();
        let handle= std::thread::spawn(move || {
            process_transaction(worker_id, receiver, clients_ref)
        });

        worker_handles.push(handle);
    }

    (worker_senders, worker_handles)
}

/// Each worker processes transactions sequentially for the particular client (see dispatcher.rs for client_id -> worker index mapping.
/// Argument worker_id is for debugging purposes
fn process_transaction(worker_id: usize, worker_receiver: Receiver<CsvTransaction>, clients: GlobalClientsMap) -> Result<(), ApplicationError> {
    for csv_transaction in worker_receiver {
        let client_id = csv_transaction.client_id;
        let tx_id = csv_transaction.tx_id;

        // println!(
        //     "[Worker {}] Processing client {} transaction {}",
        //     worker_id, client_id, tx_id
        // );

        let mut clients_map = clients.write().unwrap();
        let client = clients_map.entry(csv_transaction.client_id).or_default();

        if client.locked {
            return Err(ApplicationError::ClientAccountFrozen(client_id,tx_id));
        }

        let result = match csv_transaction.tx_type {
            TransactionType::Deposit => {
                client.deposit(&csv_transaction);
                Ok(())
            }
            TransactionType::Withdrawal => client.withdraw(&csv_transaction),
            TransactionType::Dispute => { client.dispute(csv_transaction.tx_id); Ok(()) },
            TransactionType::Resolve => { client.resolve(csv_transaction.tx_id); Ok(()) },
            TransactionType::Chargeback => { client.chargeback(csv_transaction.tx_id); Ok(()) },
        };

        if let Err(e) = result {
            eprintln!("[Worker {}] Error processing transaction {}: {}", worker_id, csv_transaction.tx_id, e);
            continue; // do not return the error as it will cause the channel to close
        }

        // println!(
        //     "[Worker {} successfully processed transaction id {}]",
        //     worker_id, csv_transaction.tx_id
        // );
    }

    Ok(())
}