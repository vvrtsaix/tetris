use tetris::multiplayer::MultiplayerServer;

#[tokio::main]
async fn main() {
    let server = MultiplayerServer::new();
    println!("Starting Tetris multiplayer server on ws://localhost:8080");
    server.start("127.0.0.1:8080").await;
} 
