use std::sync::mpsc::{Receiver, Sender};
use std::thread::JoinHandle;
use crate::client::{GlobalClientsMap};
use crate::csv_ingestor::CsvTransaction;
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
fn process_transaction(worker_id: usize, worker_receiver: Receiver<CsvTransaction>, clients: GlobalClientsMap) -> Result<(), ApplicationError> {
    for csv_transaction in worker_receiver {
        println!(
            "[Worker {}] Processing client {} transaction {}",
            worker_id, csv_transaction.client_id, csv_transaction.tx_id
        );

        let mut clients_map = clients.write().unwrap();
        let client = clients_map.entry(csv_transaction.client_id).or_default();

        // For now, just save to history
        client.tx_history.insert(csv_transaction.tx_id, csv_transaction.clone());

        println!("[Worker {} successfully processed transaction id {}", worker_id, csv_transaction.tx_id);
    }

    Ok(())
}