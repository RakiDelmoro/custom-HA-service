//! MQTT client trait and error types

use async_trait::async_trait;

/// Error type for MQTT operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum MqttError {
    ConnectionFailed(String),
    SubscribeFailed(String),
    PublishFailed(String),
    PollFailed(String),
    ConnectionRefused,
    NotConnected,
    Custom(String),
}

impl std::fmt::Display for MqttError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MqttError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            MqttError::SubscribeFailed(msg) => write!(f, "Subscribe failed: {}", msg),
            MqttError::PublishFailed(msg) => write!(f, "Publish failed: {}", msg),
            MqttError::PollFailed(msg) => write!(f, "Poll failed: {}", msg),
            MqttError::ConnectionRefused => write!(f, "Connection refused"),
            MqttError::NotConnected => write!(f, "Not connected"),
            MqttError::Custom(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for MqttError {}

impl From<rumqttc::ClientError> for MqttError {
    fn from(err: rumqttc::ClientError) -> Self {
        MqttError::Custom(err.to_string())
    }
}

/// MQTT event types for polling
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum MqttEvent {
    Connected,
    Disconnected,
    PublishReceived(String, Vec<u8>), // topic, payload
    Subscribed(String),
    Published(String),
    Other(String),
}

/// Trait for MQTT client operations
/// Allows for both real and mock implementations
#[async_trait]
pub trait MqttClient: Send + Sync {
    /// Subscribe to a topic
    async fn subscribe(&self, topic: &str) -> Result<(), MqttError>;

    /// Publish a message to a topic
    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), MqttError>;

    /// Poll for events (blocking)
    async fn poll(&mut self) -> Result<MqttEvent, MqttError>;

    /// Check if client is connected
    #[allow(dead_code)]
    fn is_connected(&self) -> bool;
}
