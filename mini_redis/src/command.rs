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
        "INCR" => incr_function(req, store),
        "DECR" => decr_function(req, store),
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

fn incr_function(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };

    let mut store = store.lock().unwrap();
    
    let new_value = match store.get_mut(&key) {
        None => {
            // clé n'existe pas → on crée avec 1
            store.insert(key, Entry { value: "1".to_string(), expires_at: None });
            1
        },
        Some(entry) => {
            // clé existe → on parse
            match entry.value.parse::<i64>() {
                Err(_) => return serde_json::json!({"status": "error", "message": "not an integer"}),
                Ok(n) => {
                    entry.value = (n + 1).to_string();
                    n + 1
                }
            }
        }
    };

    serde_json::json!({"status": "ok", "value": new_value})

}


fn decr_function(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };

    let mut store = store.lock().unwrap();
    
    let new_value = match store.get_mut(&key) {
        None => {
            // clé n'existe pas → on crée avec 1
            store.insert(key, Entry { value: "0".to_string(), expires_at: None });
            1
        },
        Some(entry) => {
            // clé existe → on parse
            match entry.value.parse::<i64>() {
                Err(_) => return serde_json::json!({"status": "error", "message": "not an integer"}),
                Ok(n) => {
                    entry.value = (n + 1).to_string();
                    n -1 
                }
            }
        }
    };

    serde_json::json!({"status": "ok", "value": new_value})

}