//! Real MQTT client implementation using rumqttc

use super::client::{MqttClient, MqttError, MqttEvent};
use async_trait::async_trait;
use rumqttc::{AsyncClient, QoS};
use std::time::Duration;

/// Real MQTT client implementation using rumqttc
pub struct RealMqttClient {
    client: AsyncClient,
}

impl RealMqttClient {
    /// Creates a new MQTT client from configuration
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use custom_ha_service::config::Config;
    /// use custom_ha_service::mqtt::RealMqttClient;
    ///
    /// async fn example() {
    ///     let config = Config::from_env();
    ///     let client = RealMqttClient::new(&config).await.unwrap();
    /// }
    /// ```
    pub async fn new(config: &crate::config::Config) -> Result<Self, MqttError> {
        let mut mqttoptions =
            rumqttc::MqttOptions::new(&config.client_id, &config.mqtt_broker, config.mqtt_port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        mqttoptions.set_clean_session(false);
        mqttoptions.set_pending_throttle(Duration::from_millis(10));

        if let (Some(username), Some(password)) = (&config.mqtt_username, &config.mqtt_password) {
            mqttoptions.set_credentials(username, password);
        }

        let (client, _eventloop) = AsyncClient::new(mqttoptions, 100);

        Ok(RealMqttClient { client })
    }

    /// Get the underlying async client (for advanced usage)
    #[allow(dead_code)]
    pub fn get_async_client(&self) -> &AsyncClient {
        &self.client
    }

    /// Subscribe to a topic (internal implementation)
    pub async fn subscribe(&self, topic: &str) -> Result<(), MqttError> {
        self.client
            .subscribe(topic, QoS::AtLeastOnce)
            .await
            .map_err(|e| MqttError::SubscribeFailed(e.to_string()))
    }

    /// Publish a message to a topic (internal implementation)
    pub async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), MqttError> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload.to_vec())
            .await
            .map_err(|e| MqttError::PublishFailed(e.to_string()))
    }
}

#[async_trait]
impl MqttClient for RealMqttClient {
    async fn subscribe(&self, topic: &str) -> Result<(), MqttError> {
        self.subscribe(topic).await
    }

    async fn publish(&self, topic: &str, payload: &[u8]) -> Result<(), MqttError> {
        self.publish(topic, payload).await
    }

    async fn poll(&mut self) -> Result<MqttEvent, MqttError> {
        // For the real client, we need to handle the eventloop separately
        // This is a placeholder - in production, eventloop.poll() is used
        // but it has complex types that don't work well with traits
        Err(MqttError::Custom(
            "RealMqttClient requires separate eventloop handling".to_string(),
        ))
    }

    fn is_connected(&self) -> bool {
        true
    }
}
