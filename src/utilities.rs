use std::fs;
use std::time::SystemTime;
use hex::encode;
use rocket::State;
use sha2::{Sha256, Digest};
use crate::core::state::AppState;

pub fn open_config(config_path: &str) -> String {
    fs::read_to_string(config_path).unwrap_or_else(|_| {
        eprintln!("Could not open your configuration");
        std::process::exit(1);
    })
}

pub fn name_parser(state: &State<AppState>, service: impl ToString) -> String {
    let service_clone = service.to_string().clone();
    let string = state.services.get(&service.to_string()).unwrap_or(&service_clone);
    string.clone().to_owned()
}

pub fn unix_time() -> u64 {
    match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(duration) => duration.as_secs(),
        Err(_) => panic!("Error while getting unix time"),
    }
}

pub fn generate_hash(string: String) -> String {
    let mut sha256 = Sha256::new();
    sha256.update(string.to_string().as_bytes());
    let result = sha256.finalize();

    encode(result)
}