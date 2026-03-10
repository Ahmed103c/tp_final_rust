use crate::commands::utils::require_key;
use crate::commands::Request;
use crate::{Entry, Store};
use serde_json::Value;

pub enum Operation {
    Incr,
    Decr,
}

pub fn incr_decr_function(req: Request, store: &Store, op: Operation) -> Value {
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
            store.insert(
                key,
                Entry {
                    value: initial.to_string(),
                    expires_at: None,
                },
            );
            initial
        }
        Some(entry) => match entry.value.parse::<i64>() {
            Err(_) => return serde_json::json!({"status": "error", "message": "not an integer"}),
            Ok(n) => {
                let new_val = match op {
                    Operation::Incr => n + 1,
                    Operation::Decr => n - 1,
                };
                entry.value = new_val.to_string();
                new_val
            }
        },
    };

    serde_json::json!({"status": "ok", "value": new_value})
}
