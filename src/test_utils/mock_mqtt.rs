//! Mock MQTT client for testing
//!
//! Provides a controllable mock implementation of the MqttClient trait
//! for testing without an actual MQTT broker.

use crate::mqtt::{MqttClient, MqttError, MqttEvent};
use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, mpsc};
use tokio::time::{sleep, Duration};

/// Mock MQTT client for testing with controllable error injection
#[allow(dead_code)]
pub struct MockMqttClient {
    /// Subscriptions: topic -> sender channel
    subscriptions: Arc<Mutex<HashMap<String, broadcast::Sender<(String, Vec<u8>)>>>>,
    
    /// All published messages: (topic, payload)
    published_messages: Arc<Mutex<Vec<(String, Vec<u8>)>>>,
    
    /// Queue of events to return from poll()
    event_queue: Arc<Mutex<VecDeque<MqttEvent>>>,
    
    /// Error injection state
    error_state: Arc<Mutex<ErrorState>>,
    
    /// Connection state
    connected: Arc<Mutex<bool>>,
    
    /// Channel for receiving injected messages (simulating incoming publishes)
    incoming_rx: Arc<Mutex<mpsc::Receiver<(String, Vec<u8>)>>>,
    incoming_tx: mpsc::Sender<(String, Vec<u8>)>,
}

impl Clone for MockMqttClient {
    fn clone(&self) -> Self {
        // Create a new channel for the cloned instance
        let (incoming_tx, incoming_rx) = mpsc::channel(100);
        
        MockMqttClient {
            subscriptions: self.subscriptions.clone(),
            published_messages: self.published_messages.clone(),
            event_queue: self.event_queue.clone(),
            error_state: self.error_state.clone(),
            connected: self.connected.clone(),
            incoming_rx: Arc::new(Mutex::new(incoming_rx)),
            incoming_tx,
        }
    }
}

#[derive(Default)]
#[allow(dead_code)]
struct ErrorState {
    /// Next subscribe will fail with this error
    next_subscribe_error: Option<MqttError>,
    
    /// Next publish will fail with this error
    next_publish_error: Option<MqttError>,
    
    /// Next poll will fail with this error
    next_poll_error: Option<MqttError>,
    
    /// Connection will fail on next operation
    connection_error: Option<MqttError>,
    
    /// Connection is currently "broken"
    is_broken: bool,
}

#[allow(dead_code)]
impl MockMqttClient {
    /// Creates a new MockMqttClient
    pub fn new() -> Self {
        let (incoming_tx, incoming_rx) = mpsc::channel(100);
        
        MockMqttClient {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            published_messages: Arc::new(Mutex::new(Vec::new())),
            event_queue: Arc::new(Mutex::new(VecDeque::new())),
            error_state: Arc::new(Mutex::new(ErrorState::default())),
            connected: Arc::new(Mutex::new(true)),
            incoming_rx: Arc::new(Mutex::new(incoming_rx)),
            incoming_tx,
        }
    }

    /// Inject an error on the next subscribe call
    pub fn inject_subscribe_error(&self, error: MqttError) {
        self.error_state.lock().unwrap().next_subscribe_error = Some(error);
    }

    /// Inject an error on the next publish call
    pub fn inject_publish_error(&self, error: MqttError) {
        self.error_state.lock().unwrap().next_publish_error = Some(error);
    }

    /// Inject an error on the next poll call
    pub fn inject_poll_error(&self, error: MqttError) {
        self.error_state.lock().unwrap().next_poll_error = Some(error);
    }

    /// Simulate a connection failure
    pub fn inject_connection_failure(&self, error: MqttError) {
        let mut state = self.error_state.lock().unwrap();
        state.connection_error = Some(error);
        *self.connected.lock().unwrap() = false;
    }

    /// Break the connection (simulates network failure)
    pub fn break_connection(&self) {
        *self.error_state.lock().unwrap() = ErrorState {
            is_broken: true,
            ..Default::default()
        };
        *self.connected.lock().unwrap() = false;
    }

    /// Restore the connection (simulates network recovery)
    pub fn restore_connection(&self) {
        *self.error_state.lock().unwrap() = ErrorState::default();
        *self.connected.lock().unwrap() = true;
    }

    /// Clear all error injections
    pub fn clear_errors(&self) {
        *self.error_state.lock().unwrap() = ErrorState::default();
    }

    /// Inject a message as if it was received from the broker
    pub fn inject_incoming_message(&self, topic: &str, payload: &[u8]) {
        let _ = self.incoming_tx.try_send((topic.to_string(), payload.to_vec()));
    }

    /// Schedule an event to be returned from poll()
    pub fn schedule_event(&self, event: MqttEvent) {
        self.event_queue.lock().unwrap().push_back(event);
    }

    /// Get all published messages
    pub fn get_published_messages(&self) -> Vec<(String, Vec<u8>)> {
        self.published_messages.lock().unwrap().clone()
    }

    /// Get messages published to a specific topic
    pub fn get_messages_for_topic(&self, topic: &str) -> Vec<Vec<u8>> {
        self.published_messages
            .lock()
            .unwrap()
            .iter()
            .filter(|(t, _)| t == topic)
            .map(|(_, payload)| payload.clone())
            .collect()
    }

    /// Check if a message was published containing specific content
    pub fn has_published_message(&self, topic: &str, content: &str) -> bool {
        self.published_messages
            .lock()
            .unwrap()
            .iter()
            .filter(|(t, _)| t == topic)
            .any(|(_, payload)| {
                String::from_utf8_lossy(payload).contains(content)
            })
    }

    /// Get the last published message for a topic
    pub fn get_last_message(&self, topic: &str) -> Option<Vec<u8>> {
        self.published_messages
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|(t, _)| t == topic)
            .map(|(_, payload)| payload.clone())
    }

    /// Clear all published messages
    pub fn clear_messages(&self) {
        self.published_messages.lock().unwrap().clear();
    }

    /// Get connection state
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }

    /// Subscribe and get a receiver for this topic
    pub fn subscribe_and_receive(&self, topic: &str) -> broadcast::Receiver<(String, Vec<u8>)> {
        let mut subs = self.subscriptions.lock().unwrap();
        
        if let Some(tx) = subs.get(topic) {
            tx.subscribe()
        } else {
            let (tx, rx) = broadcast::channel(100);
            subs.insert(topic.to_string(), tx);
            rx
        }
    }
}

#[async_trait]
impl MqttClient for MockMqttClient {
    async fn subscribe(&self, topic: &str) -> Result<(), MqttError> {
        // Check for injected errors
        if let Some(error) = self.error_state.lock().unwrap().next_subscribe_error.take() {
            return Err(error);
        }

        if self.error_state.lock().unwrap().is_broken {
            return Err(MqttError::NotConnected);
        }

        // Create subscription channel if not exists
        let mut subs = self.subscriptions.lock().unwrap();
        if !subs.contains_key(topic) {
            let (tx, _) = broadcast::channel(100);
            subs.insert(topic.to_string(), tx);
        }

        Ok(())
    }

    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), MqttError> {
        // Check for injected errors
        if let Some(error) = self.error_state.lock().unwrap().next_publish_error.take() {
            return Err(error);
        }

        if self.error_state.lock().unwrap().is_broken {
            return Err(MqttError::NotConnected);
        }

        // Record the published message
        self.published_messages
            .lock()
            .unwrap()
            .push((topic.to_string(), payload.to_vec()));

        // Broadcast to subscribers
        let subs = self.subscriptions.lock().unwrap();
        if let Some(tx) = subs.get(topic) {
            let _ = tx.send((topic.to_string(), payload.to_vec()));
        }

        Ok(())
    }

    async fn poll(&mut self) -> Result<MqttEvent, MqttError> {
        // Check for injected errors
        if let Some(error) = self.error_state.lock().unwrap().next_poll_error.take() {
            return Err(error);
        }

        if self.error_state.lock().unwrap().is_broken {
            return Err(MqttError::ConnectionRefused);
        }

        // Check scheduled events first
        if let Some(event) = self.event_queue.lock().unwrap().pop_front() {
            return Ok(event);
        }

        // Check for incoming messages
        if let Ok((topic, payload)) = self.incoming_rx.lock().unwrap().try_recv() {
            return Ok(MqttEvent::PublishReceived(topic, payload));
        }

        // No events, wait a bit then return a keepalive
        sleep(Duration::from_millis(10)).await;
        Ok(MqttEvent::Other("Keepalive".to_string()))
    }

    fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap()
    }
}

impl Default for MockMqttClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_subscribe_and_publish() {
        let mock = MockMqttClient::new();
        
        mock.subscribe("test/topic").await.unwrap();
        mock.publish("test/topic", b"Hello, World!").await.unwrap();
        
        let messages = mock.get_messages_for_topic("test/topic");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], b"Hello, World!");
    }

    #[tokio::test]
    async fn test_error_injection() {
        let mock = MockMqttClient::new();
        
        mock.inject_publish_error(MqttError::NotConnected);
        
        let result = mock.publish("test/topic", b"data").await;
        assert!(matches!(result, Err(MqttError::NotConnected)));
        
        // Next publish should succeed
        mock.publish("test/topic", b"data").await.unwrap();
    }

    #[tokio::test]
    async fn test_connection_break_and_restore() {
        let mock = MockMqttClient::new();
        
        assert!(mock.is_connected());
        
        mock.break_connection();
        assert!(!mock.is_connected());
        
        let result = mock.publish("test/topic", b"data").await;
        assert!(matches!(result, Err(MqttError::NotConnected)));
        
        mock.restore_connection();
        assert!(mock.is_connected());
        
        mock.publish("test/topic", b"data").await.unwrap();
    }

    #[tokio::test]
    async fn test_inject_incoming_message() {
        let mut mock = MockMqttClient::new();
        
        mock.inject_incoming_message("input/topic", b"test payload");
        
        let event = mock.poll().await.unwrap();
        match event {
            MqttEvent::PublishReceived(topic, payload) => {
                assert_eq!(topic, "input/topic");
                assert_eq!(payload, b"test payload");
            }
            _ => panic!("Expected PublishReceived event"),
        }
    }
}
