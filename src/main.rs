use std::{env, thread};
use std::sync::mpsc::channel;
use std::thread::JoinHandle;
use drizzly::client::{new_clients_map};
use drizzly::error::ApplicationError;
use drizzly::csv_ingestor::{read_csv, CsvTransaction};
use drizzly::dispatcher::Dispatcher;
use drizzly::worker::spawn_workers;

fn main() -> Result<(), ApplicationError>{
    // make sure 1 argument provided to the program which should be the CSV path
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <path_to_csv>");
        std::process::exit(1);
    }

    let csv_path = &args[1];

    // initialize empty map of Clients
    let global_clients_map = new_clients_map();

    // Spawn worker threads responsible for processing a specific clients' transactions
    let (worker_senders, worker_handles) = spawn_workers(global_clients_map.clone());

    // Create dispatcher channel
    // Dispatcher is responsible for finding the worker assigned to client_id
    // and sending the csv transaction that worker
    let (dispatcher_sender, ingestion_receiver) = channel();
    let dispatcher = Dispatcher::new(worker_senders);
    dispatcher.start(ingestion_receiver);

    // Spawn CSV ingestion thread
    let csv_path_clone = csv_path.to_string();
    let ingestion_handle:JoinHandle<Result<Vec<CsvTransaction>, ApplicationError>> = thread::spawn(move || {
        read_csv(&csv_path_clone, dispatcher_sender)
    });

    // wait till data ingestion is done
    match ingestion_handle.join() {
        Ok(Ok(_)) => { println!("CSV ingestion thread finished!");}
        Ok(Err(e)) => return Err(e), // propagate ApplicationError
        Err(panic) => panic!("CSV ingestion thread panicked: {:?}", panic),
    }

    // wait till workers are done after
    for handle in worker_handles {
        match handle.join() {
            Ok(Ok(())) => { println!("worker successfully finished and shutdown!"); },
            Ok(Err(e)) => return Err(e),
            Err(panic) => {
                return Err(ApplicationError::Other(format!("Worker thread panicked: {:?}", panic)));
            }
        }
    }

    println!("Process all transactions from {csv_path}!!!");
    Ok(())
}
