use std::env;
use drizzly::error::ApplicationError;
use drizzly::ingestion::read_csv;

fn main() -> Result<(), ApplicationError>{
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <path_to_csv>");
        std::process::exit(1);
    }

    let csv_path = &args[1];
    let transactions = read_csv(csv_path)?;
    println!("Deserialized {} transactions", transactions.len());
    Ok(())
}
