use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;
use futures_util::{SinkExt, StreamExt};

#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerState {
    pub player_id: String,
    pub score: i32,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum GameMessage {
    Join { player_id: String },
    GameState { player_id: String, score: i32 },
    LineCleared { player_id: String, count: i32 },
    GameOver { player_id: String },
    PlayerLeft { player_id: String },
}

type Clients = Arc<Mutex<HashMap<String, mpsc::UnboundedSender<Message>>>>;
type PlayerStates = Arc<Mutex<HashMap<String, PlayerState>>>;

pub struct MultiplayerServer {
    clients: Clients,
    player_states: PlayerStates,
}

impl MultiplayerServer {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            player_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&self, addr: &str) {
        let listener = TcpListener::bind(addr).await.expect("Failed to bind");
        println!("WebSocket server listening on: {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream.peer_addr().expect("Connected streams should have a peer address");
            println!("Peer address: {}", peer);

            let clients = self.clients.clone();
            let player_states = self.player_states.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, clients, player_states).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }

    async fn handle_connection(
        stream: TcpStream,
        clients: Clients,
        player_states: PlayerStates,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ws_stream = tokio_tungstenite::accept_async(stream).await?;
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (outgoing_tx, mut outgoing_rx) = mpsc::unbounded_channel();

        // Generate player ID
        let player_id = uuid::Uuid::new_v4().to_string();
        
        // Send Join message to the new player
        let join_msg = GameMessage::Join {
            player_id: player_id.clone(),
        };
        ws_sender.send(Message::Text(serde_json::to_string(&join_msg)?)).await?;

        // Add new player to states and get current states
        let current_states = {
            let mut states = player_states.lock().unwrap();
            states.insert(player_id.clone(), PlayerState {
                player_id: player_id.clone(),
                score: 0,
                name: None,
            });
            states.values().cloned().collect::<Vec<_>>()
        };

        // Store the sender in clients map
        {
            let mut clients_guard = clients.lock().unwrap();
            clients_guard.insert(player_id.clone(), tx.clone());
        }

        // Send current player states to new player
        for state in current_states {
            let msg = GameMessage::GameState {
                player_id: state.player_id,
                score: state.score,
            };
            ws_sender.send(Message::Text(serde_json::to_string(&msg)?)).await?;
        }

        // Broadcast new player joined to all other clients
        {
            let broadcast_join = Message::Text(serde_json::to_string(&join_msg)?);
            let clients_guard = clients.lock().unwrap();
            for (id, client) in clients_guard.iter() {
                if *id != player_id {
                    let _ = client.send(broadcast_join.clone());
                }
            }
        }

        // Handle outgoing messages to WebSocket
        let outgoing_handle = tokio::spawn(async move {
            while let Some(msg) = outgoing_rx.recv().await {
                if let Err(e) = ws_sender.send(msg).await {
                    eprintln!("WebSocket send error: {}", e);
                    break;
                }
            }
        });

        // Handle incoming messages from other clients
        let incoming_handle = tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                if let Err(e) = outgoing_tx.send(msg) {
                    eprintln!("Channel send error: {}", e);
                    break;
                }
            }
        });

        // Handle messages from the WebSocket
        while let Some(result) = ws_receiver.next().await {
            match result {
                Ok(msg) => {
                    if let Ok(game_msg) = serde_json::from_str::<GameMessage>(&msg.to_string()) {
                        // Update player state
                        if let GameMessage::GameState { player_id, score } = &game_msg {
                            let mut states = player_states.lock().unwrap();
                            if let Some(state) = states.get_mut(player_id) {
                                state.score = *score;
                            }
                            drop(states);
                        }

                        // Broadcast the message to all other clients
                        let broadcast_msg = Message::Text(serde_json::to_string(&game_msg)?);
                        let clients_guard = clients.lock().unwrap();
                        for (id, client) in clients_guard.iter() {
                            if *id != player_id {
                                let _ = client.send(broadcast_msg.clone());
                            }
                        }
                        drop(clients_guard);
                    }
                }
                Err(e) => {
                    eprintln!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        // Clean up when client disconnects
        {
            let mut clients_guard = clients.lock().unwrap();
            clients_guard.remove(&player_id);
        }
        {
            let mut states = player_states.lock().unwrap();
            states.remove(&player_id);
        }

        // Broadcast player left message
        let left_msg = GameMessage::PlayerLeft {
            player_id: player_id.clone(),
        };
        let broadcast_msg = Message::Text(serde_json::to_string(&left_msg)?);
        {
            let clients_guard = clients.lock().unwrap();
            for client in clients_guard.values() {
                let _ = client.send(broadcast_msg.clone());
            }
        }

        // Clean up tasks
        outgoing_handle.abort();
        incoming_handle.abort();

        Ok(())
    }
}

pub struct MultiplayerClient {
    sender: mpsc::UnboundedSender<GameMessage>,
    receiver: mpsc::UnboundedReceiver<GameMessage>,
}

impl MultiplayerClient {
    pub async fn connect(server_addr: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(server_addr).await?;
        let (mut write, mut read) = ws_stream.split();
        
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (msg_tx, msg_rx) = mpsc::unbounded_channel();

        // Handle incoming messages
        tokio::spawn(async move {
            while let Some(msg) = read.next().await {
                if let Ok(msg) = msg {
                    if let Ok(game_msg) = serde_json::from_str(&msg.to_string()) {
                        let _ = msg_tx.send(game_msg);
                    }
                }
            }
        });

        // Handle outgoing messages
        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let json = serde_json::to_string(&msg).unwrap();
                let _ = write.send(Message::Text(json)).await;
            }
        });

        Ok(Self {
            sender: tx,
            receiver: msg_rx,
        })
    }

    pub fn send(&self, msg: GameMessage) {
        let _ = self.sender.send(msg);
    }

    pub fn try_receive(&mut self) -> Option<GameMessage> {
        self.receiver.try_recv().ok()
    }
} 
