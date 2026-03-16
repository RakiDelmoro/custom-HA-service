use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub mqtt_broker: String,
    pub mqtt_port: u16,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
    pub subscribe_topic: String,
    pub client_id: String,
    pub publish_topic: String,
    pub pulses_per_liter: u32,
}

impl Config {
    pub fn from_env() -> Self {
        Config {
            mqtt_broker: env::var("MQTT_BROKER").unwrap_or_else(|_| "homeassistant".to_string()),
            mqtt_port: env::var("MQTT_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(1883),
            mqtt_username: env::var("MQTT_USERNAME").ok(),
            mqtt_password: env::var("MQTT_PASSWORD").ok(),
            subscribe_topic: env::var("SUBSCRIBE_TOPIC")
                .unwrap_or_else(|_| "custom-service/in".to_string()),
            client_id: env::var("CLIENT_ID").unwrap_or_else(|_| "flowpulse-mqtt".to_string()),
            publish_topic: env::var("PUBLISH_TOPIC")
                .unwrap_or_else(|_| "flowpulse/out".to_string()),
            pulses_per_liter: env::var("PULSES_PER_LITER")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(433),
        }
    }
}
