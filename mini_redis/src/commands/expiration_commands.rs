use crate::commands::utils::{require_key, require_seconds};
use crate::commands::Request;
use crate::Store;
use serde_json::Value;
use std::time::{Duration, Instant};

/// Returns all keys currently present in the store.
/// Order is not guaranteed.
pub fn keys_function(store: &Store) -> Value {
    let store = store.lock().unwrap();
    let keys: Vec<String> = store.keys().cloned().collect();
    serde_json::json!({"status": "ok", "keys": keys})
}

/// Associates an expiration time (in seconds) with an existing key.
/// Once the delay has elapsed, the key will be automatically removed.
/// Returns an error if the key does not exist.
pub fn expire_function(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let seconds = match require_seconds(&req) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let mut store = store.lock().unwrap();
    if let Some(entry) = store.get_mut(&key) {
        entry.expires_at = Some(Instant::now() + Duration::from_secs(seconds));
        serde_json::json!({"status": "ok"})
    } else {
        serde_json::json!({"status": "error", "message": "key not found"})
    }
}

/// Returns the remaining time to live of a key in seconds.
/// - Positive integer : seconds remaining before expiration
/// - `-1` : key exists but has no expiration
/// - `-2` : key does not exist
pub fn ttl_function(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let store = store.lock().unwrap();
    match store.get(&key) {
        None => serde_json::json!({"status": "ok", "ttl": -2}),
        Some(entry) => match entry.expires_at {
            None => serde_json::json!({"status": "ok", "ttl": -1}),
            Some(expires_at) => {
                let ttl = expires_at.duration_since(Instant::now()).as_secs();
                serde_json::json!({"status": "ok", "ttl": ttl})
            }
        },
    }
}