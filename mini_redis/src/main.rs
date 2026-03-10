use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

mod command; 
use command::get_command_response;
  
type Store = Arc<Mutex<HashMap<String, String>>>;

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

