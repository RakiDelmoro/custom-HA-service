mod config;
mod models;
mod processor;
mod state;

use rumqttc::{AsyncClient, ConnAck, Event, EventLoop, Incoming, MqttOptions, Outgoing, Publish, QoS, SubAck};
use std::time::Duration;
use tokio::time::sleep;
use log::{info, error, debug, warn};

use config::Config;
use models::SensorData;
use processor::process_sensor_data;
use state::{create_state, current_time_ms, SharedState};

async fn create_mqtt_client(config: &Config) -> (AsyncClient, EventLoop) {
    let mut mqttoptions = MqttOptions::new(&config.client_id, &config.mqtt_broker, config.mqtt_port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    mqttoptions.set_clean_session(false);
    mqttoptions.set_pending_throttle(Duration::from_millis(10));
    
    if let (Some(username), Some(password)) = (&config.mqtt_username, &config.mqtt_password) {
        mqttoptions.set_credentials(username, password);
    }

    AsyncClient::new(mqttoptions, 100)
}

async fn subscribe_to_topic(client: &AsyncClient, topic: &str) -> Result<(), rumqttc::ClientError> {
    client.subscribe(topic, QoS::AtLeastOnce).await
}

async fn handle_message(
    client: &AsyncClient,
    publish: &Publish,
    config: &Config,
    state: SharedState,
) -> Result<(), rumqttc::ClientError> {
    let topic = publish.topic.clone();
    let payload_str = String::from_utf8_lossy(&publish.payload);
    
    // Get receive timestamp
    let receive_time_ms = current_time_ms();
    
    debug!("Received message on {}: {}", topic, payload_str);
    
    // Parse JSON payload
    let sensor_data: SensorData = match serde_json::from_str(&payload_str) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to parse sensor data: {}", e);
            return Ok(());
        }
    };
    
    info!(
        "Processing sensor data: {} pulses over {} ms",
        sensor_data.total_pulses, sensor_data.time_ms
    );
    
    // Process the data
    let output = {
        let mut state_guard = state.lock().unwrap();
        state_guard.update_last_receive(receive_time_ms);
        process_sensor_data(
            sensor_data,
            &mut state_guard,
            config.pulses_per_liter,
            receive_time_ms,
        )
    };
    
    // Filter out timestamps that have already been published (Question 3 = B: Skip existing)
    let entries_to_publish: Vec<_> = {
        let mut state_guard = state.lock().unwrap();
        output.timeseries
            .into_iter()
            .filter(|entry| {
                if state_guard.is_timestamp_published(&entry.timestamp) {
                    debug!("Skipping already published timestamp: {}", entry.timestamp);
                    false
                } else {
                    state_guard.mark_timestamp_published(&entry.timestamp);
                    true
                }
            })
            .collect()
    };
    
    // Skip if all timestamps already published
    if entries_to_publish.is_empty() {
        info!("All timestamps already published, skipping");
        return Ok(());
    }
    
    // Serialize: always as array
    let response = match serde_json::to_string(&entries_to_publish) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize output: {}", e);
            return Ok(());
        }
    };
    
    // Publish to MQTT
    client.publish(&config.publish_topic, QoS::AtLeastOnce, false, response.clone()).await?;
    
    info!(
        "Published {} entries ({} gap-filled with zeros) to {}",
        entries_to_publish.len(),
        output.metadata.zero_entries_count,
        config.publish_topic
    );
    
    // Log the first and last entries for verification
    if let Some(first) = entries_to_publish.first() {
        debug!("First entry: {} = {} L/min", first.timestamp, first.flow_rate_lpm);
    }
    if let Some(last) = entries_to_publish.last() {
        debug!("Last entry: {} = {} L/min", last.timestamp, last.flow_rate_lpm);
    }
    
    debug!("Published data: {}", response);
    
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let config = Config::from_env();
    let state = create_state();
    
    info!("Starting FlowPulse MQTT");
    info!("MQTT broker: {}:{}", config.mqtt_broker, config.mqtt_port);
    info!("Subscribe topic: {}", config.subscribe_topic);
    info!("Publish topic: {}", config.publish_topic);
    info!("Pulses per liter: {}", config.pulses_per_liter);
    
    loop {
        info!("Connecting to MQTT broker...");
        let (client, mut eventloop) = create_mqtt_client(&config).await;
        
        // Subscribe to the configured topic
        match subscribe_to_topic(&client, &config.subscribe_topic).await {
            Ok(_) => info!("Subscribed to topic: {}", config.subscribe_topic),
            Err(e) => {
                error!("Failed to subscribe to topic: {}", e);
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        }

        info!("Connected and subscribed. Waiting for sensor data...");
        
        // Main event loop
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Incoming::Publish(publish))) => {
                    if let Err(e) = handle_message(&client, &publish, &config, state.clone()).await {
                        error!("Error handling message: {}", e);
                    }
                }
                Ok(Event::Incoming(Incoming::ConnAck(ConnAck { session_present, .. }))) => {
                    if !session_present {
                        // Re-subscribe if session is not present
                        if let Err(e) = subscribe_to_topic(&client, &config.subscribe_topic).await {
                            error!("Failed to re-subscribe: {}", e);
                            break;
                        }
                    }
                }
                Ok(Event::Incoming(Incoming::SubAck(SubAck { pkid, .. }))) => {
                    debug!("Subscription {} acknowledged", pkid);
                }
                Ok(Event::Outgoing(Outgoing::Publish(pkid))) => {
                    debug!("Publish {} sent", pkid);
                }
                Ok(Event::Outgoing(Outgoing::Subscribe(pkid))) => {
                    debug!("Subscribe {} sent", pkid);
                }
                Ok(_) => {
                    // Other events, ignore
                }
                Err(e) => {
                    error!("Connection error: {:?}", e);
                    break;
                }
            }
        }
        
        warn!("Disconnected from MQTT broker. Reconnecting in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}