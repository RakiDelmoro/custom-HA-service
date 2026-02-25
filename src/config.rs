use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub mqtt_broker: String,
    pub mqtt_port: u16,
    pub mqtt_username: Option<String>,
    pub mqtt_password: Option<String>,
    pub subscribe_topic: String,
    pub publish_topic: String,
    pub client_id: String,
    pub pulses_per_liter: u32,
    pub gap_fill_mode: GapFillMode,
    pub gap_threshold_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GapFillMode {
    LastKnown,
    Zero,
}

impl Config {
    pub fn from_env() -> Self {
        let gap_fill_mode = match env::var("GAP_FILL_MODE")
            .unwrap_or_else(|_| "last".to_string())
            .to_lowercase()
            .as_str()
        {
            "zero" => GapFillMode::Zero,
            _ => GapFillMode::LastKnown,
        };

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
            publish_topic: env::var("PUBLISH_TOPIC")
                .unwrap_or_else(|_| "custom-service/out".to_string()),
            client_id: env::var("CLIENT_ID").unwrap_or_else(|_| "custom-ha-service".to_string()),
            pulses_per_liter: env::var("PULSES_PER_LITER")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(433),
            gap_fill_mode,
            gap_threshold_ms: env::var("GAP_THRESHOLD_MS")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(2000),
        }
    }
}
