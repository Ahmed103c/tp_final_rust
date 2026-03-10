use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::test]
async fn test_ping() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();
    
    let command = serde_json::json!({"cmd": "PING"});
    let mut command_str = command.to_string();
    command_str.push('\n');
    stream.write_all(command_str.as_bytes()).await.unwrap();
    
    let mut reader = BufReader::new(&mut stream);
    let mut response = String::new();
    reader.read_line(&mut response).await.unwrap();
    
    let json: Value = serde_json::from_str(response.trim()).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_set() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();
    
    let command = serde_json::json!({"cmd": "SET", "key": "test", "value": "hello"});
    let mut command_str = command.to_string();
    command_str.push('\n');
    stream.write_all(command_str.as_bytes()).await.unwrap();
    
    let mut reader = BufReader::new(&mut stream);
    let mut response = String::new();
    reader.read_line(&mut response).await.unwrap();
    
    let json: Value = serde_json::from_str(response.trim()).unwrap();
    assert_eq!(json["status"], "ok");
}