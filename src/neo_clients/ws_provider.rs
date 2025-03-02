use crate::neo_clients::{JsonRpcProvider, ProviderError};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio_tungstenite::{
    connect_async, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};
use url::Url;
use uuid::Uuid;

/// WebSocket provider for Neo N3 blockchain interactions
pub struct WsProvider {
    /// The WebSocket endpoint URL
    url: Url,
    /// Sender for WebSocket messages
    message_tx: Sender<Message>,
    /// Map of subscription IDs to response channels
    subscriptions: Arc<Mutex<HashMap<String, Sender<Value>>>>,
    /// Connection ID (used for reconnection)
    connection_id: String,
}

/// Neo N3 WebSocket subscription
pub struct Subscription<T> {
    /// The subscription ID
    id: String,
    /// Channel for receiving subscription events
    receiver: Receiver<Value>,
    /// Type marker
    _marker: std::marker::PhantomData<T>,
}

impl<T: for<'de> Deserialize<'de>> Subscription<T> {
    /// Create a new subscription
    fn new(id: String, receiver: Receiver<Value>) -> Self {
        Self {
            id,
            receiver,
            _marker: std::marker::PhantomData,
        }
    }

    /// Get the next event from the subscription
    pub async fn next(&mut self) -> Option<T> {
        if let Some(value) = self.receiver.recv().await {
            match serde_json::from_value(value) {
                Ok(result) => Some(result),
                Err(err) => {
                    log::error!("Failed to deserialize subscription event: {}", err);
                    None
                }
            }
        } else {
            None
        }
    }
}

impl WsProvider {
    /// Create a new WebSocket provider
    pub async fn connect(url: &str) -> Result<Self, ProviderError> {
        let url = Url::parse(url).map_err(|e| ProviderError::ConnectionError(e.to_string()))?;
        
        // Connect to the WebSocket server
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| ProviderError::ConnectionError(e.to_string()))?;
        
        // Create channels for message passing
        let (message_tx, mut message_rx) = mpsc::channel::<Message>(100);
        let subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let subscriptions_clone = Arc::clone(&subscriptions);
        let connection_id = Uuid::new_v4().to_string();
        
        // Start background task to handle WebSocket messages
        tokio::spawn(async move {
            handle_websocket(ws_stream, message_rx, subscriptions_clone).await;
        });
        
        Ok(Self {
            url,
            message_tx,
            subscriptions,
            connection_id,
        })
    }
    
    /// Subscribe to blocks
    pub async fn subscribe_blocks(&self) -> Result<Subscription<Block>, ProviderError> {
        // Create subscription
        let (tx, rx) = mpsc::channel(100);
        let subscription_id = format!("blocks_{}", Uuid::new_v4());
        
        // Register subscription
        {
            let mut subs = self.subscriptions.lock().unwrap();
            subs.insert(subscription_id.clone(), tx);
        }
        
        // Send subscription request
        let request = json!({
            "jsonrpc": "2.0",
            "method": "subscribe",
            "params": ["block"],
            "id": subscription_id.clone()
        });
        
        self.message_tx
            .send(Message::Text(request.to_string()))
            .await
            .map_err(|e| ProviderError::ConnectionError(e.to_string()))?;
        
        Ok(Subscription::new(subscription_id, rx))
    }
    
    /// Subscribe to contract notifications
    pub async fn subscribe_contract_notifications(
        &self,
        contract_hash: &str,
    ) -> Result<Subscription<Notification>, ProviderError> {
        // Create subscription
        let (tx, rx) = mpsc::channel(100);
        let subscription_id = format!("notifications_{}_{}", contract_hash, Uuid::new_v4());
        
        // Register subscription
        {
            let mut subs = self.subscriptions.lock().unwrap();
            subs.insert(subscription_id.clone(), tx);
        }
        
        // Send subscription request
        let request = json!({
            "jsonrpc": "2.0",
            "method": "subscribe",
            "params": ["notifications", { "contract": contract_hash }],
            "id": subscription_id.clone()
        });
        
        self.message_tx
            .send(Message::Text(request.to_string()))
            .await
            .map_err(|e| ProviderError::ConnectionError(e.to_string()))?;
        
        Ok(Subscription::new(subscription_id, rx))
    }
    
    /// Subscribe to transactions
    pub async fn subscribe_transactions(&self) -> Result<Subscription<Transaction>, ProviderError> {
        // Create subscription
        let (tx, rx) = mpsc::channel(100);
        let subscription_id = format!("transactions_{}", Uuid::new_v4());
        
        // Register subscription
        {
            let mut subs = self.subscriptions.lock().unwrap();
            subs.insert(subscription_id.clone(), tx);
        }
        
        // Send subscription request
        let request = json!({
            "jsonrpc": "2.0",
            "method": "subscribe",
            "params": ["transaction"],
            "id": subscription_id.clone()
        });
        
        self.message_tx
            .send(Message::Text(request.to_string()))
            .await
            .map_err(|e| ProviderError::ConnectionError(e.to_string()))?;
        
        Ok(Subscription::new(subscription_id, rx))
    }
    
    /// Unsubscribe from a subscription
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<bool, ProviderError> {
        // Remove subscription
        {
            let mut subs = self.subscriptions.lock().unwrap();
            subs.remove(subscription_id);
        }
        
        // Send unsubscribe request
        let request = json!({
            "jsonrpc": "2.0",
            "method": "unsubscribe",
            "params": [subscription_id],
            "id": Uuid::new_v4().to_string()
        });
        
        self.message_tx
            .send(Message::Text(request.to_string()))
            .await
            .map_err(|e| ProviderError::ConnectionError(e.to_string()))?;
        
        Ok(true)
    }
}

impl JsonRpcProvider for WsProvider {
    async fn request<T: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: Value,
    ) -> Result<T, ProviderError> {
        let id = Uuid::new_v4().to_string();
        let (tx, mut rx) = mpsc::channel(1);
        
        // Register one-time request
        {
            let mut subs = self.subscriptions.lock().unwrap();
            subs.insert(id.clone(), tx);
        }
        
        // Create JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": id.clone()
        });
        
        // Send request
        self.message_tx
            .send(Message::Text(request.to_string()))
            .await
            .map_err(|e| ProviderError::ConnectionError(e.to_string()))?;
        
        // Wait for response
        match rx.recv().await {
            Some(response) => {
                // Remove one-time request
                {
                    let mut subs = self.subscriptions.lock().unwrap();
                    subs.remove(&id);
                }
                
                if let Some(error) = response.get("error") {
                    return Err(ProviderError::RpcError(error.to_string()));
                }
                
                match response.get("result") {
                    Some(result) => {
                        serde_json::from_value(result.clone())
                            .map_err(|e| ProviderError::DeserializationError(e.to_string()))
                    }
                    None => Err(ProviderError::InvalidResponse("No result field".to_string())),
                }
            }
            None => Err(ProviderError::ConnectionError("No response received".to_string())),
        }
    }
}

/// Handle WebSocket messages in a background task
async fn handle_websocket(
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
    mut message_rx: Receiver<Message>,
    subscriptions: Arc<Mutex<HashMap<String, Sender<Value>>>>,
) {
    let (mut write, mut read) = ws_stream.split();
    
    loop {
        tokio::select! {
            // Handle outgoing messages
            Some(message) = message_rx.recv() => {
                if let Err(e) = write.send(message).await {
                    log::error!("Error sending message: {}", e);
                    break;
                }
            }
            
            // Handle incoming messages
            Some(message_result) = read.next() => {
                match message_result {
                    Ok(message) => {
                        if let Message::Text(text) = message {
                            let value: Value = match serde_json::from_str(&text) {
                                Ok(v) => v,
                                Err(e) => {
                                    log::error!("Failed to parse message: {}", e);
                                    continue;
                                }
                            };
                            
                            // Handle subscription messages
                            if let Some(method) = value.get("method") {
                                if method.as_str() == Some("subscription") {
                                    if let Some(params) = value.get("params") {
                                        if let Some(subscription_id) = params.get("subscription").and_then(|s| s.as_str()) {
                                            let subs = subscriptions.lock().unwrap();
                                            if let Some(sender) = subs.get(subscription_id) {
                                                if let Some(result) = params.get("result") {
                                                    let _ = sender.try_send(result.clone());
                                                }
                                            }
                                        }
                                    }
                                    continue;
                                }
                            }
                            
                            // Handle regular responses
                            if let Some(id) = value.get("id").and_then(|id| id.as_str()) {
                                let subs = subscriptions.lock().unwrap();
                                if let Some(sender) = subs.get(id) {
                                    let _ = sender.try_send(value.clone());
                                }
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error receiving message: {}", e);
                        break;
                    }
                }
            }
            
            else => break,
        }
    }
    
    log::warn!("WebSocket connection closed");
}

// For type-safety in the subscription API
use crate::neo_types::{Block, Notification, Transaction};

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;
    use std::time::Duration;

    #[tokio::test]
    async fn test_connect() {
        let url = "wss://testnet1.neo.org:60002/ws";  // Replace with a valid Neo N3 WebSocket endpoint
        let provider = WsProvider::connect(url).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_subscribe_blocks() {
        let url = "wss://testnet1.neo.org:60002/ws";  // Replace with a valid Neo N3 WebSocket endpoint
        if let Ok(provider) = WsProvider::connect(url).await {
            let subscription = provider.subscribe_blocks().await;
            assert!(subscription.is_ok());
            
            // Wait for at most 10 seconds for a block
            if let Ok(mut subscription) = subscription {
                let result = timeout(Duration::from_secs(10), subscription.next()).await;
                
                // We don't assert the result as we might not get a block in the test timeframe
                // This is just to check that the subscription works without errors
                println!("Block subscription result: {:?}", result);
            }
        }
    }
} 