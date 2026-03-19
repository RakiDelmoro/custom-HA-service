use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

/// In-memory MQTT mock for testing without external broker
/// Simulates MQTT publish/subscribe behavior using channels
pub struct InMemoryMqttMock {
    subscriptions: Arc<Mutex<HashMap<String, broadcast::Sender<String>>>>,
    published_messages: Arc<Mutex<Vec<(String, String)>>>, // (topic, payload)
}

impl InMemoryMqttMock {
    pub fn new() -> Self {
        InMemoryMqttMock {
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
            published_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Subscribe to a topic and return a receiver for messages
    pub fn subscribe(&self, topic: &str) -> broadcast::Receiver<String> {
        let mut subs = self.subscriptions.lock().unwrap();

        // If topic already has a sender, use it to create a new receiver
        if let Some(tx) = subs.get(topic) {
            return tx.subscribe();
        }

        // Otherwise create new channel
        let (tx, rx) = broadcast::channel(100);
        subs.insert(topic.to_string(), tx);
        rx
    }

    /// Publish a message to a topic
    pub fn publish(&self, topic: &str, payload: &str) -> Result<(), String> {
        // Record the published message
        self.published_messages
            .lock()
            .unwrap()
            .push((topic.to_string(), payload.to_string()));

        // Send to all subscribers
        let subs = self.subscriptions.lock().unwrap();
        if let Some(tx) = subs.get(topic) {
            if tx.send(payload.to_string()).is_err() {
                // No subscribers, that's ok
            }
        }
        Ok(())
    }

    /// Get all published messages
    pub fn get_published_messages(&self) -> Vec<(String, String)> {
        self.published_messages.lock().unwrap().clone()
    }

    /// Clear published messages
    pub fn clear_messages(&self) {
        self.published_messages.lock().unwrap().clear();
    }
}

impl Default for InMemoryMqttMock {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_in_memory_mock_pub_sub() {
        let mock = InMemoryMqttMock::new();

        let mut rx = mock.subscribe("test/topic");

        mock.publish("test/topic", r#"{"data": "value"}"#).unwrap();

        // Wait a bit for message propagation
        sleep(Duration::from_millis(10)).await;

        let received = rx.try_recv();
        assert!(received.is_ok());
        assert_eq!(received.unwrap(), r#"{"data": "value"}"#);

        let messages = mock.get_published_messages();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].0, "test/topic");
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let mock = InMemoryMqttMock::new();

        let mut rx1 = mock.subscribe("shared/topic");
        let mut rx2 = mock.subscribe("shared/topic");

        mock.publish("shared/topic", "broadcast message").unwrap();

        sleep(Duration::from_millis(10)).await;

        let msg1 = rx1.try_recv().unwrap();
        let msg2 = rx2.try_recv().unwrap();

        assert_eq!(msg1, "broadcast message");
        assert_eq!(msg2, "broadcast message");
    }

    #[tokio::test]
    async fn test_no_subscriber_no_error() {
        let mock = InMemoryMqttMock::new();

        // Publishing without subscribers should not error
        let result = mock.publish("empty/topic", "message");
        assert!(result.is_ok());

        // Message should still be recorded
        let messages = mock.get_published_messages();
        assert_eq!(messages.len(), 1);
    }
}
