use crate::commands::Request;
use serde_json::Value;

pub fn require_key(req: &Request) -> Result<String, Value> {
    match req.key.clone() {
        Some(k) => Ok(k),
        None => Err(serde_json::json!({"status": "error", "message": "missing key"})),
    }
}

pub fn require_seconds(req: &Request) -> Result<u64, Value> {
    match req.seconds {
        Some(s) => Ok(s),
        None => Err(serde_json::json!({"status": "error", "message": "missing seconds"})),
    }
}
