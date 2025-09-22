use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
};
use tokio::time::interval;
use uuid::Uuid;

use crate::types::{DefiProtocolStats, YieldOpportunity};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "price_update")]
    PriceUpdate {
        token: String,
        price: f64,
        change_24h: f64,
        timestamp: u64,
    },
    #[serde(rename = "portfolio_update")]
    PortfolioUpdate {
        address: String,
        total_value: f64,
        change_24h: f64,
        timestamp: u64,
    },
    #[serde(rename = "protocol_stats")]
    ProtocolStats {
        protocol: String,
        stats: DefiProtocolStats,
        timestamp: u64,
    },
    #[serde(rename = "yield_opportunities")]
    YieldOpportunities {
        opportunities: Vec<YieldOpportunity>,
        timestamp: u64,
    },
    #[serde(rename = "transaction_update")]
    TransactionUpdate {
        hash: String,
        status: String,
        confirmation_count: u32,
        timestamp: u64,
    },
    #[serde(rename = "security_alert")]
    SecurityAlert {
        level: String,
        title: String,
        description: String,
        timestamp: u64,
    },
    #[serde(rename = "connection")]
    Connection {
        client_id: String,
        message: String,
        timestamp: u64,
    },
    #[serde(rename = "error")]
    Error {
        code: String,
        message: String,
        timestamp: u64,
    },
}

#[derive(Debug, Clone)]
pub struct WebSocketClient {
    pub id: String,
    pub subscriptions: Vec<String>,
    pub sender: tokio::sync::mpsc::UnboundedSender<WebSocketMessage>,
}

pub type WebSocketClients = Arc<RwLock<HashMap<String, WebSocketClient>>>;

pub struct WebSocketState {
    pub clients: WebSocketClients,
}

impl WebSocketState {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn broadcast(&self, message: WebSocketMessage) {
        let clients = self.clients.read().unwrap().clone();
        for client in clients.values() {
            if let Err(_) = client.sender.send(message.clone()) {
                // Client disconnected, will be cleaned up
            }
        }
    }

    pub async fn send_to_client(&self, client_id: &str, message: WebSocketMessage) {
        let clients = self.clients.read().unwrap();
        if let Some(client) = clients.get(client_id) {
            let _ = client.sender.send(message);
        }
    }

    pub fn add_client(&self, client: WebSocketClient) {
        let mut clients = self.clients.write().unwrap();
        clients.insert(client.id.clone(), client);
    }

    pub fn remove_client(&self, client_id: &str) {
        let mut clients = self.clients.write().unwrap();
        clients.remove(client_id);
    }

    pub fn get_client_count(&self) -> usize {
        self.clients.read().unwrap().len()
    }
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<crate::api::ApiState>>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, Arc::clone(&state.websocket)))
}

async fn handle_socket(socket: WebSocket, state: Arc<WebSocketState>) {
    let client_id = Uuid::new_v4().to_string();
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WebSocketMessage>();

    // Create client
    let client = WebSocketClient {
        id: client_id.clone(),
        subscriptions: vec!["prices".to_string(), "portfolio".to_string()],
        sender: tx,
    };

    // Add client to state
    state.add_client(client);

    // Send welcome message
    let welcome_msg = WebSocketMessage::Connection {
        client_id: client_id.clone(),
        message: "Connected to blockchain demo WebSocket".to_string(),
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    if let Err(_) = sender
        .send(Message::Text(serde_json::to_string(&welcome_msg).unwrap()))
        .await
    {
        return;
    }

    println!("WebSocket client connected: {}", client_id);

    // Spawn task to handle outgoing messages
    let state_clone = Arc::clone(&state);
    let client_id_clone = client_id.clone();
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let msg_text = match serde_json::to_string(&msg) {
                Ok(text) => text,
                Err(_) => continue,
            };

            if sender.send(Message::Text(msg_text)).await.is_err() {
                break;
            }
        }
        
        // Clean up disconnected client
        state_clone.remove_client(&client_id_clone);
        println!("WebSocket client disconnected: {}", client_id_clone);
    });

    // Handle incoming messages
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    // Handle subscription requests
                    if let Some(subscription) = parsed.get("subscribe") {
                        if let Some(topic) = subscription.as_str() {
                            println!("Client {} subscribed to: {}", client_id, topic);
                            // Add subscription logic here
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("WebSocket connection closed: {}", client_id);
                break;
            }
            _ => {}
        }
    }

    // Clean up client
    state.remove_client(&client_id);
}

// Background task to simulate real-time updates
pub async fn start_real_time_updates(state: Arc<WebSocketState>) {
    let mut interval = interval(Duration::from_secs(5)); // Update every 5 seconds

    tokio::spawn(async move {
        loop {
            interval.tick().await;

            // Simulate price updates
            let price_update = WebSocketMessage::PriceUpdate {
                token: "ETH".to_string(),
                price: 1750.0 + (rand::random::<f64>() - 0.5) * 50.0,
                change_24h: (rand::random::<f64>() - 0.5) * 10.0,
                timestamp: chrono::Utc::now().timestamp() as u64,
            };

            state.broadcast(price_update).await;

            // Simulate protocol stats updates
            let protocol_stats = WebSocketMessage::ProtocolStats {
                protocol: "aave".to_string(),
                stats: DefiProtocolStats {
                    name: "Aave".to_string(),
                    tvl: "$5.2B".to_string(),
                    total_borrowed: "$3.1B".to_string(),
                    total_supplied: "$8.3B".to_string(),
                    utilization_rate: 40.5 + (rand::random::<f64>() - 0.5) * 5.0,
                    average_supply_apy: 3.5 + (rand::random::<f64>() - 0.5) * 1.0,
                    average_borrow_apy: 5.2 + (rand::random::<f64>() - 0.5) * 1.5,
                    active_users: 45000 + rand::random::<u32>() % 1000,
                    health_factor: 2.1 + (rand::random::<f64>() - 0.5) * 0.3,
                },
                timestamp: chrono::Utc::now().timestamp() as u64,
            };

            state.broadcast(protocol_stats).await;

            // Log active connections
            let client_count = state.get_client_count();
            if client_count > 0 {
                println!("Broadcasting updates to {} clients", client_count);
            }
        }
    });
}

// Helper function to send security alerts
pub async fn send_security_alert(
    state: Arc<WebSocketState>,
    level: String,
    title: String,
    description: String,
) {
    let alert = WebSocketMessage::SecurityAlert {
        level,
        title,
        description,
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    state.broadcast(alert).await;
}

// Helper function to send transaction updates
pub async fn send_transaction_update(
    state: Arc<WebSocketState>,
    hash: String,
    status: String,
    confirmation_count: u32,
) {
    let update = WebSocketMessage::TransactionUpdate {
        hash,
        status,
        confirmation_count,
        timestamp: chrono::Utc::now().timestamp() as u64,
    };

    state.broadcast(update).await;
}