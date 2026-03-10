use std::collections::HashMap;
use std::time::Duration;
use std::time::Instant;

use crate::Store;
use crate::Entry;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Request {
    pub cmd: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub seconds: Option<u64>
}

enum Operation {
    Incr,
    Decr,
}

pub fn get_command_response(line: &str, store: &Store) -> Value {
    let req: Request = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(_) => return serde_json::json!({"status": "error", "message": "invalid json"}),
    };

    match req.cmd.as_str() {
        "PING" => serde_json::json!({"status": "ok"}),
        "SET" => set_function(req, store),
        "GET" => get_function(req, store),
        "DEL" => del_function(req, store),
        "KEYS" => keys_function(store),
        "EXPIRE" => expire_function(req, store),
        "TTL" => ttl_function(req, store),
        "INCR" => incr_decr_function(req, store, Operation::Incr),
        "DECR" => incr_decr_function(req, store, Operation::Decr),
        "SAVE" => save_function(store),
        _ => serde_json::json!({"status": "error", "message": "unknown command"}),
    }
}

fn require_key(req: &Request) -> Result<String, Value> {
    match req.key.clone() {
        Some(k) => Ok(k),
        None => Err(serde_json::json!({"status": "error", "message": "missing key"})),
    }
}

fn require_seconds(req: &Request) -> Result<u64, Value> {
    match req.seconds {
        Some(s) => Ok(s),
        None => Err(serde_json::json!({"status": "error", "message": "missing seconds"})),
    }
}


fn set_function(req: Request, store: &Store) -> Value {
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

fn get_function(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let value = store.lock().unwrap().get(&key).map(|e| e.value.clone());
    serde_json::json!({"status": "ok", "value": value})
}

fn del_function(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let count = if store.lock().unwrap().remove(&key).is_some() { 1 } else { 0 };
    serde_json::json!({"status": "ok", "count": count})
}

fn keys_function(store: &Store) -> Value {
    let store = store.lock().unwrap();
    let keys: Vec<String> = store.keys().cloned().collect();
    serde_json::json!({"status": "ok", "keys": keys})
}

fn expire_function(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let seconds = match require_seconds(&req){
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

fn ttl_function(req: Request, store: &Store) -> Value {
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
        }
    }
}

fn incr_decr_function(req: Request, store: &Store, op: Operation) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };

    let mut store = store.lock().unwrap();
    
    let new_value = match store.get_mut(&key) {
        None => {
            let initial = match op {
                Operation::Incr => 1,
                Operation::Decr => -1,
            };
            store.insert(key, Entry { value: initial.to_string(), expires_at: None });
            initial
        },
        Some(entry) => {
            match entry.value.parse::<i64>() {
                Err(_) => return serde_json::json!({"status": "error", "message": "not an integer"}),
                Ok(n) => {
                    let new_val = match op {
                        Operation::Incr => n + 1,
                        Operation::Decr => n - 1,
                    };
                    entry.value = new_val.to_string();
                    new_val
                }
            }
        }
    };

    serde_json::json!({"status": "ok", "value": new_value})
}

fn save_function(store: &Store) -> Value {
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
        }
    }
}