pub mod basic_commands;
pub mod expiration_commands;
pub mod value_commands;
pub mod save_command;
pub mod utils;

use crate::Store;
use serde::Deserialize;
use serde_json::Value;

/// Represents a JSON request received from a client.
/// All fields except `cmd` are optional depending on the command.
#[derive(Deserialize)]
pub struct Request {
    pub cmd: String,
    pub key: Option<String>,
    pub value: Option<String>,
    pub seconds: Option<u64>,
}

/// Parses a JSON line and routes it to the appropriate command handler.
/// Returns an error response if the JSON is invalid or the command is unknown.
pub fn get_command_response(line: &str, store: &Store) -> Value {
    let req: Request = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(_) => return serde_json::json!({"status": "error", "message": "invalid json"}),
    };

    match req.cmd.as_str() {
        "PING" => serde_json::json!({"status": "ok"}),
        "SET"  => basic_commands::set_function(req, store),
        "GET"  => basic_commands::get_function(req, store),
        "DEL"  => basic_commands::del_function(req, store),
        "KEYS"   => expiration_commands::keys_function(store),
        "EXPIRE" => expiration_commands::expire_function(req, store),
        "TTL"    => expiration_commands::ttl_function(req, store),
        "INCR" => value_commands::incr_decr_function(req, store, value_commands::Operation::Incr),
        "DECR" => value_commands::incr_decr_function(req, store, value_commands::Operation::Decr),
        "SAVE" => save_command::save_function(store),
        _ => serde_json::json!({"status": "error", "message": "unknown command"}),
    }
}