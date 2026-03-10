use crate::Store;
use serde_json::Value;
use std::collections::HashMap;

pub fn save_function(store: &Store) -> Value {
    let store = store.lock().unwrap();
    let map: HashMap<String, String> = store
        .iter()
        .map(|(k, v)| (k.clone(), v.value.clone()))
        .collect();

    match serde_json::to_string(&map) {
        Err(_) => serde_json::json!({"status": "error", "message": "failed to serialize"}),
        Ok(json) => match std::fs::write("dump.json", json) {
            Err(_) => serde_json::json!({"status": "error", "message": "failed to write file"}),
            Ok(_) => serde_json::json!({"status": "ok"}),
        },
    }
}
