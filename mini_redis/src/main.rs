use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

mod commands;
use commands::get_command_response;

/// A key-value store entry.
/// Contains the value and an optional expiration time.
#[derive(Clone)]
pub struct Entry {
    pub value: String,
    pub expires_at: Option<Instant>,
}

/// Shared store between all Tokio tasks.
/// Arc allows sharing, Mutex protects concurrent access.
pub type Store = Arc<Mutex<HashMap<String, Entry>>>;

/// Server listening address
const ADDR: &str = "127.0.0.1:7878";

/// Cleanup interval for expired keys (in seconds)
const CLEANUP_INTERVAL_SECS: u64 = 1;

#[tokio::main]
async fn main() {
    init_tracing();

    let store: Store = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind(ADDR).await.unwrap();

    tracing::info!("MiniRedis listening on {}", ADDR);

    spawn_cleanup_task(store.clone());
    accept_loop(listener, store).await;
}

/// Initializes the logging system with tracing.
/// Uses RUST_LOG if set, otherwise defaults to "info".
fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();
}

/// Spawns a background task that removes expired keys
/// every `CLEANUP_INTERVAL_SECS` seconds.
fn spawn_cleanup_task(store: Store) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));
        loop {
            interval.tick().await;
            store.lock().unwrap().retain(|_, entry| match entry.expires_at {
                Some(expires_at) => expires_at > Instant::now(),
                None => true,
            });
        }
    });
}

/// Main TCP connection acceptance loop.
/// Each connection is handled in an independent Tokio task.
async fn accept_loop(listener: TcpListener, store: Store) {
    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        tracing::info!("New connection: {}", addr);
        let store = store.clone();
        tokio::spawn(async move {
            answer_client(socket, store).await;
        });
    }
}

/// Handles a connected client — reads JSON commands line by line
/// and sends back JSON responses.
async fn answer_client(socket: TcpStream, store: Store) {
    let (read_half, mut write_half) = socket.into_split();
    let mut lines = BufReader::new(read_half).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let response = get_command_response(&line, &store);
        let mut response_str = serde_json::to_string(&response).unwrap();
        response_str.push('\n');
        if write_half.write_all(response_str.as_bytes()).await.is_err() {
            break;
        }
    }
}