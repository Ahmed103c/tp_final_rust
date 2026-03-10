use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

#[tokio::test]
async fn test_ping() {
    let mut stream = TcpStream::connect("127.0.0.1:7878").await.unwrap();
    
    stream.write_all(b"{\"cmd\": \"PING\"}\n").await.unwrap();
    
    let mut reader = BufReader::new(&mut stream);
    let mut response = String::new();
    reader.read_line(&mut response).await.unwrap();
    
    assert_eq!(response.trim(), "{\"status\":\"ok\"}");
}