use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use serde::{Deserialize, Serialize};
use serde_json::Value;

type Store = Arc<Mutex<HashMap<String, String>>>;

#[derive(Deserialize)]
struct Request {
    cmd: String,
    key: Option<String>,
    value: Option<String>,
}

#[tokio::main]
async fn main() {
    // Initialiser tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // TODO: Implémenter le serveur MiniRedis sur 127.0.0.1:7878
    //
    // Étapes suggérées :
    // 1. Créer le store partagé (Arc<Mutex<HashMap<String, ...>>>)
    let store: Store = Arc::new(Mutex::new(HashMap::new()));
    // 2. Bind un TcpListener sur 127.0.0.1:7878
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    // 3. Accept loop : pour chaque connexion, spawn une tâche
    // 4. Dans chaque tâche : lire les requêtes JSON ligne par ligne,
    //    traiter la commande, envoyer la réponse JSON + '\n'

    
    tracing::info!("MiniRedis listening on 127.0.0.1:7878");

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        tracing::info!("New connection: {}", addr);
        let store = store.clone();
        tokio::spawn(async move {
            answer_client(socket, store).await;
        });
    }
}

async fn answer_client(socket: tokio::net::TcpStream, store: Store) {

    let (read_half, mut write_half) = socket.into_split();

    let reader = BufReader::new(read_half);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let response = get_command_response(&line, &store);
        let mut response_str = serde_json::to_string(&response).unwrap();
        response_str.push('\n');
        if write_half.write_all(response_str.as_bytes()).await.is_err() {
            break;
        }
    }
}


fn get_command_response(line: &str, store: &Store) -> Value {
    let req: Request = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(_) => return serde_json::json!({"status": "error", "message": "invalid json"}),
    };

    match req.cmd.as_str() {
        "PING" => serde_json::json!({"status": "ok"}),

        "SET" => {
            let key = match req.key {
                Some(k) => k,
                None => return serde_json::json!({"status": "error", "message": "missing key"}),
            };
            let value = match req.value {
                Some(v) => v,
                None => return serde_json::json!({"status": "error", "message": "missing value"}),
            };
            let mut store = store.lock().unwrap();
            store.insert(key, value);
            serde_json::json!({"status": "ok"})
        },

        "GET" => {
            let key = match req.key {
                Some(k) => k,
                None => return serde_json::json!({"status": "error", "message": "missing key"}),
            };
            let store = store.lock().unwrap();
            let value = store.get(&key).cloned();
            serde_json::json!({"status": "ok", "value": value})
        },
        
        "DEL" => {
            let key = match req.key {
                Some(k) => k,
                None => return serde_json::json!({"status": "error", "message": "missing key"}),
            };
            let mut store = store.lock().unwrap();
            let count = if store.remove(&key).is_some() { 1 } else { 0 };
            serde_json::json!({"status": "ok", "count": count})
        },
        _ => serde_json::json!({"status": "error", "message": "unknown command"}),
    }
}
