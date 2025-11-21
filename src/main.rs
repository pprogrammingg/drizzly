use std::{env, thread};
use std::sync::mpsc::channel;
use drizzly::client::{new_clients_map};
use drizzly::error::ApplicationError;
use drizzly::csv_ingestor::{read_csv};
use drizzly::dispatcher::Dispatcher;
use drizzly::worker::spawn_workers;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <path_to_csv>");
        std::process::exit(1);
    }

    let csv_path = &args[1];

    // Create shared clients map
    let global_clients_map = new_clients_map();

    // Error accumulator
    let mut errors_list: Vec<ApplicationError> = Vec::new();

    // Spawn workers
    let (worker_senders, worker_handles) = spawn_workers(global_clients_map.clone());

    // Create dispatcher
    let (dispatcher_sender, ingestion_receiver) = channel();
    let dispatcher = Dispatcher::new(worker_senders);
    let dispatcher_handle = dispatcher.start(ingestion_receiver);

    // Spawn CSV ingestion
    let csv_path_clone = csv_path.to_string();
    let ingestion_handle = thread::spawn(move || {
        read_csv(&csv_path_clone, dispatcher_sender)
    });

    // Wait on CSV thread
    match ingestion_handle.join() {
        Ok(Ok(_)) => eprintln!("CSV ingestion thread finished"),
        Ok(Err(e)) => errors_list.push(e),
        Err(panic) => errors_list.push(
            ApplicationError::Other(format!("CSV ingestion panic: {:?}", panic))
        ),
    }

    // Wait on dispatcher
    match dispatcher_handle.join() {
        Ok(Ok(())) => eprintln!("Dispatcher terminated"),
        Ok(Err(e)) => errors_list.push(e),
        Err(panic) => errors_list.push(
            ApplicationError::Other(format!("Dispatcher panic: {:?}", panic))
        ),
    }

    // Wait on workers
    for handle in worker_handles {
        match handle.join() {
            Ok(Ok(())) => eprintln!("Worker terminated"),
            Ok(Err(e)) => errors_list.push(e),
            Err(panic) => errors_list.push(
                ApplicationError::Other(format!("Worker panic: {:?}", panic))
            ),
        }
    }

    // print global accounts to STD output
    println!("client,available,held,total,locked");

    // unlock RWLock
    let clients_guard = global_clients_map.read().unwrap();
    for (id, client) in clients_guard.iter() {
        // output all amounts in 4 decimal places
        println!("{},{:.4},{:.4},{:.4},{}",
                 id, client.available, client.held, client.total, client.locked
        );
    }
    drop(clients_guard);

    // print error to STD error
    if !errors_list.is_empty() {
        eprintln!("Errors encountered during processing:");
        for e in errors_list {
            eprintln!(" - {}", e);
        }
    }
}