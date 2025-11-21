use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::csv_ingestor::CsvTransaction;


/// Clients - a thread-safe mutable hashmap which holds client-id vs state


#[derive(Debug, Default)]
pub struct Client {
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,

    pub tx_history: HashMap<u32, CsvTransaction>,
}

pub type GlobalClientsMap = Arc<RwLock<HashMap<u16, Client>>>;

pub fn new_clients_map() -> GlobalClientsMap {
    Arc::new(RwLock::new(HashMap::new()))
}
