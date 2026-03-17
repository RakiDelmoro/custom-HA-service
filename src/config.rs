//! Configuration management
//!
//! This module handles all configuration for the FlowPulse service.
//! Configuration is loaded from environment variables with sensible defaults.
//!
//! # Environment Variables
//!
//! | Variable | Default | Description |
//! |----------|---------|-------------|
//! | `MQTT_BROKER` | `homeassistant` | MQTT broker host/IP |
//! | `MQTT_PORT` | `1883` | MQTT broker port |
//! | `MQTT_USERNAME` | - | Optional username |
//! | `MQTT_PASSWORD` | - | Optional password |
//! | `SUBSCRIBE_TOPIC` | `custom-service/in` | Input topic |
//! | `PUBLISH_TOPIC` | `flowpulse/out` | Output topic |
//! | `CLIENT_ID` | `flowpulse-mqtt` | MQTT client ID |
//! | `PULSES_PER_LITER` | `433` | Sensor pulses per liter |
//!
//! # Example
//!
//! ```bash
//! MQTT_BROKER=192.168.1.100 \
//! MQTT_PORT=1883 \
//! SUBSCRIBE_TOPIC=sensor/watermeter \
//! PUBLISH_TOPIC=home/water/flow \
//! PULSES_PER_LITER=1000 \
//! ./custom-ha-service
//! ```

use std::env;

/// Service configuration
///
/// All fields are public for easy access throughout the codebase.
/// Use `Config::from_env()` to load from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// MQTT broker host or IP address
    pub mqtt_broker: String,
    /// MQTT broker port
    pub mqtt_port: u16,
    /// Optional MQTT username
    pub mqtt_username: Option<String>,
    /// Optional MQTT password
    pub mqtt_password: Option<String>,
    /// Topic to subscribe for sensor input
    pub subscribe_topic: String,
    /// MQTT client ID
    pub client_id: String,
    /// Topic to publish flow rate output
    pub publish_topic: String,
    /// Number of pulses per liter for flow calculation
    pub pulses_per_liter: u32,
}

impl Config {
    /// Creates a new Config with values from environment variables
    ///
    /// Uses sensible defaults for all values. In production, you should
    /// set at least `MQTT_BROKER` to point to your MQTT broker.
    ///
    /// # Example
    ///
    /// ```rust
    /// use custom_ha_service::config::Config;
    ///
    /// let config = Config::from_env();
    /// println!("Connecting to broker: {}:{}", config.mqtt_broker, config.mqtt_port);
    /// ```
    pub fn from_env() -> Self {
        Config {
            mqtt_broker: env::var("MQTT_BROKER").unwrap_or_else(|_| "homeassistant".to_string()),
            mqtt_port: parse_port("MQTT_PORT", 1883),
            mqtt_username: env::var("MQTT_USERNAME").ok(),
            mqtt_password: env::var("MQTT_PASSWORD").ok(),
            subscribe_topic: env::var("SUBSCRIBE_TOPIC")
                .unwrap_or_else(|_| "custom-service/in".to_string()),
            client_id: env::var("CLIENT_ID").unwrap_or_else(|_| "flowpulse-mqtt".to_string()),
            publish_topic: env::var("PUBLISH_TOPIC")
                .unwrap_or_else(|_| "flowpulse/out".to_string()),
            pulses_per_liter: parse_u32("PULSES_PER_LITER", 433),
        }
    }

    /// Creates a new Config for testing
    ///
    /// This creates a config with test-friendly defaults.
    #[cfg(test)]
    pub fn test_config() -> Self {
        Config {
            mqtt_broker: "test-broker".to_string(),
            mqtt_port: 1883,
            mqtt_username: None,
            mqtt_password: None,
            subscribe_topic: "test/in".to_string(),
            client_id: "test-client".to_string(),
            publish_topic: "test/out".to_string(),
            pulses_per_liter: 433,
        }
    }

    /// Validates the configuration
    ///
    /// Returns `Ok(())` if valid, or an error message if invalid.
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), String> {
        if self.mqtt_broker.is_empty() {
            return Err("MQTT_BROKER cannot be empty".to_string());
        }
        if self.subscribe_topic.is_empty() {
            return Err("SUBSCRIBE_TOPIC cannot be empty".to_string());
        }
        if self.publish_topic.is_empty() {
            return Err("PUBLISH_TOPIC cannot be empty".to_string());
        }
        if self.pulses_per_liter == 0 {
            return Err("PULSES_PER_LITER must be greater than 0".to_string());
        }
        Ok(())
    }

    /// Returns the MQTT connection string for display/logging
    #[allow(dead_code)]
    pub fn connection_string(&self) -> String {
        format!("{}:{}", self.mqtt_broker, self.mqtt_port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Helper function to parse port from environment variable
fn parse_port(var: &str, default: u16) -> u16 {
    env::var(var)
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(default)
}

/// Helper function to parse u32 from environment variable
fn parse_u32(var: &str, default: u32) -> u32 {
    env::var(var)
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::from_env();
        assert!(!config.mqtt_broker.is_empty());
        assert!(config.mqtt_port > 0);
        assert!(!config.subscribe_topic.is_empty());
        assert!(!config.publish_topic.is_empty());
        assert!(config.pulses_per_liter > 0);
    }

    #[test]
    fn test_config_validation() {
        let config = Config::test_config();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_broker() {
        let mut config = Config::test_config();
        config.mqtt_broker = "".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_zero_pulses() {
        let mut config = Config::test_config();
        config.pulses_per_liter = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_connection_string() {
        let config = Config::test_config();
        assert_eq!(config.connection_string(), "test-broker:1883");
    }
}
