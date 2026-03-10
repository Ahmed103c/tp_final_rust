use crate::Store;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct Request {
    pub cmd: String,
    pub key: Option<String>,
    pub value: Option<String>,
}

pub fn get_command_response(line: &str, store: &Store) -> Value {
    let req: Request = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(_) => return serde_json::json!({"status": "error", "message": "invalid json"}),
    };

    match req.cmd.as_str() {
        "PING" => serde_json::json!({"status": "ok"}),
        "SET" => cmd_set(req, store),
        "GET" => cmd_get(req, store),
        "DEL" => cmd_del(req, store),
        _ => serde_json::json!({"status": "error", "message": "unknown command"}),
    }
}

fn require_key(req: &Request) -> Result<String, Value> {
    match req.key.clone() {
        Some(k) => Ok(k),
        None => Err(serde_json::json!({"status": "error", "message": "missing key"})),
    }
}

fn cmd_set(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let value = match req.value {
        Some(v) => v,
        None => return serde_json::json!({"status": "error", "message": "missing value"}),
    };
    store.lock().unwrap().insert(key, value);
    serde_json::json!({"status": "ok"})
}

fn cmd_get(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let value = store.lock().unwrap().get(&key).cloned();
    serde_json::json!({"status": "ok", "value": value})
}

fn cmd_del(req: Request, store: &Store) -> Value {
    let key = match require_key(&req) {
        Ok(k) => k,
        Err(e) => return e,
    };
    let count = if store.lock().unwrap().remove(&key).is_some() { 1 } else { 0 };
    serde_json::json!({"status": "ok", "count": count})
}