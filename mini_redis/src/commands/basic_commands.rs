use crate::commands::utils::require_key;
use crate::commands::Request;
use crate::{Entry, Store};
use serde_json::Value;

/// Stores a key-value pair in the store.
/// If the key already exists, its value is overwritten.
pub fn set_function(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let value = match req.value {
        Some(v) => v,
        None => return serde_json::json!({"status": "error", "message": "missing value"}),
    };
    store.lock().unwrap().insert(key, Entry { value, expires_at: None });
    serde_json::json!({"status": "ok"})
}

/// Returns the value associated with a key.
/// If the key does not exist, returns `"value": null`.
pub fn get_function(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let value = store.lock().unwrap().get(&key).map(|e| e.value.clone());
    serde_json::json!({"status": "ok", "value": value})
}

/// Deletes a key from the store.
/// Returns `"count": 1` if the key existed, `"count": 0` otherwise.
pub fn del_function(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let count = if store.lock().unwrap().remove(&key).is_some() { 1 } else { 0 };
    serde_json::json!({"status": "ok", "count": count})
}