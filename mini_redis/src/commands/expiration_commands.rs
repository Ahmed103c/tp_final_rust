use crate::commands::utils::{require_key, require_seconds};
use crate::commands::Request;
use crate::Store;
use serde_json::Value;
use std::time::{Duration, Instant};

pub fn keys_function(store: &Store) -> Value {
    let store = store.lock().unwrap();
    let keys: Vec<String> = store.keys().cloned().collect();
    serde_json::json!({"status": "ok", "keys": keys})
}

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
