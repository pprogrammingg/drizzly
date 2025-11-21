
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use crate::csv_ingestor::CsvTransaction;

/// Dispatcher forwards transactions to a worker assigned specifically to a client id
pub struct Dispatcher {
    worker_senders: Vec<Sender<CsvTransaction>>,
    num_workers: usize,
}

impl Dispatcher {
    pub fn new(worker_senders: Vec<Sender<CsvTransaction>>) -> Self {
        let num_workers = worker_senders.len();
        Self { worker_senders, num_workers }
    }

    /// Deterministic assignment of client to a worker
    /// This way transaction order for a particular is gauranteed while worker takes client request
    fn assign_worker(&self, client_id: u16) -> usize {
        (client_id as usize) % self.num_workers
    }

    /// Start dispatcher loop in its own thread, select the right worker based on client_id
    /// and send transactions to it
    pub fn start(self, ingestion_receiver: Receiver<CsvTransaction>) {
        thread::spawn(move || {
            for csv_transaction in ingestion_receiver {
                let worker_index = self.assign_worker(csv_transaction.client_id);

                if let Err(e) = self.worker_senders[worker_index].send(csv_transaction) {
                    eprintln!("Dispatcher failed to send to worker {}: {}", worker_index, e);
                }
            }
        });
    }
}
