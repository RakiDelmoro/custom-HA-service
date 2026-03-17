//! FlowPulse MQTT Service - Binary Entry Point
//!
//! This is the main entry point for the FlowPulse MQTT service.
//! It loads configuration from environment variables and starts the service.

use custom_ha_service::config::Config;
use custom_ha_service::mqtt::RealMqttClient;
use custom_ha_service::service::FlowPulseService;
use custom_ha_service::time::SystemTimeProvider;
use log::info;

#[tokio::main]
async fn main() {
    // Initialize logging
    simple_logger::init_with_level(log::Level::Info).unwrap();

    // Load configuration from environment
    let config = Config::from_env();
    
    // Log startup information
    info!("Starting FlowPulse MQTT Service v{}", custom_ha_service::version());
    info!("MQTT broker: {}:{}", config.mqtt_broker, config.mqtt_port);
    info!("Subscribe topic: {}", config.subscribe_topic);
    info!("Publish topic: {}", config.publish_topic);
    info!("Pulses per liter: {}", config.pulses_per_liter);

    // Create MQTT client
    let client = match RealMqttClient::new(&config).await {
        Ok(client) => client,
        Err(e) => {
            log::error!("Failed to create MQTT client: {}", e);
            std::process::exit(1);
        }
    };

    // Create service and time provider
    let service = FlowPulseService::new(config);
    let time_provider = SystemTimeProvider::new();

    // Run the service (this runs forever)
    if let Err(e) = service.run(client, time_provider).await {
        log::error!("Service error: {}", e);
        std::process::exit(1);
    }
}
