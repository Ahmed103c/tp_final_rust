use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

async fn send_command(stream: &mut TcpStream, command: Value) {
    let mut command_str = command.to_string();
    command_str.push('\n');
    stream.write_all(command_str.as_bytes()).await.unwrap();
}

async fn read_response(stream: &mut TcpStream) -> Value {
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).await.unwrap();
    serde_json::from_str(response.trim()).unwrap()
}

#[tokio::test]
async fn test_ping() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();
    send_command(&mut stream, serde_json::json!({"cmd": "PING"})).await;
    let json = read_response(&mut stream).await;
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_set() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();
    send_command(&mut stream, serde_json::json!({"cmd": "SET", "key": "test", "value": "hello"})).await;
    let json = read_response(&mut stream).await;
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_get_without_value() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();
    send_command(&mut stream, serde_json::json!({"cmd": "GET", "key": "inexistant"})).await;
    let json = read_response(&mut stream).await;
    assert_eq!(json["status"], "ok");
    assert_eq!(json["value"], Value::Null);
}

#[tokio::test]
async fn test_get_with_value() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();
    send_command(&mut stream, serde_json::json!({"cmd": "SET", "key": "test", "value": "hello"})).await;
    let _ = read_response(&mut stream).await;
    send_command(&mut stream, serde_json::json!({"cmd": "GET", "key": "test"})).await;
    let json = read_response(&mut stream).await;
    assert_eq!(json["status"], "ok");
    assert_eq!(json["value"], "hello");
}

#[tokio::test]
async fn test_del_existing_key() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();
    
    //Setting Key
    send_command(&mut stream, serde_json::json!({"cmd": "SET", "key": "to_delete", "value": "hello"})).await;
    let _ = read_response(&mut stream).await;
    
    //Deleting Key
    send_command(&mut stream, serde_json::json!({"cmd": "DEL", "key": "to_delete"})).await;
    let json = read_response(&mut stream).await;
    assert_eq!(json["status"], "ok");
    assert_eq!(json["count"], 1);
}

#[tokio::test]
async fn test_del_non_existing_key() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();
    
    send_command(&mut stream, serde_json::json!({"cmd": "DEL", "key": "inexistant"})).await;
    let json = read_response(&mut stream).await;
    assert_eq!(json["status"], "ok");
    assert_eq!(json["count"], 0);
}