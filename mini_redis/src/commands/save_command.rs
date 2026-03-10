use crate::Store;
use serde_json::Value;
use std::collections::HashMap;

/// Path to the dump file for persistent storage.
const DUMP_FILE: &str = "dump.json";

/// Saves the current store state to `dump.json`.
/// Only key-value pairs are saved, expiration times are not persisted.
/// Returns an error response if serialization or file writing fails.
pub fn save_function(store: &Store) -> Value {
    let store = store.lock().unwrap();
    let map: HashMap<String, String> = store
        .iter()
        .map(|(k, v)| (k.clone(), v.value.clone()))
        .collect();

    match serde_json::to_string(&map) {
        Err(_) => serde_json::json!({"status": "error", "message": "failed to serialize"}),
        Ok(json) => match std::fs::write(DUMP_FILE, json) {
            Err(_) => serde_json::json!({"status": "error", "message": "failed to write file"}),
            Ok(_) => serde_json::json!({"status": "ok"}),
        },
    }
}