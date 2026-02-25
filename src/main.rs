mod config;
mod models;
mod processor;
mod state;

use rumqttc::{AsyncClient, ConnAck, Event, EventLoop, Incoming, MqttOptions, Outgoing, Publish, QoS, SubAck};
use std::time::Duration;
use tokio::time::{interval, sleep};
use log::{info, error, debug, warn};
use std::sync::atomic::{AtomicBool, Ordering};

use config::Config;
use models::SensorData;
use processor::{process_sensor_data, generate_zero_entry};
use state::{create_state, current_time_ms, SharedState};

// Flag to track if we should publish zeros (set to false when data received, true after processing)
static SHOULD_PUBLISH_ZEROS: AtomicBool = AtomicBool::new(true);

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

async fn publish_zero_entry(
    client: &AsyncClient, 
    topic: &str,
    state: SharedState,
) -> Result<(), rumqttc::ClientError> {
    let zero_entry = generate_zero_entry();
    
    // Check for duplicate timestamp
    let should_publish = {
        let mut state_guard = state.lock().unwrap();
        state_guard.should_publish_zero(&zero_entry.timestamp)
    };
    
    if !should_publish {
        debug!("Skipping duplicate zero entry for timestamp: {}", zero_entry.timestamp);
        return Ok(());
    }
    
    // Serialize just the entry (not wrapped in OutputMessage)
    let response = match serde_json::to_string(&zero_entry) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize zero entry: {}", e);
            return Ok(());
        }
    };
    
    client.publish(topic, QoS::AtLeastOnce, false, response).await?;
    info!("Published zero flow entry: {}", zero_entry.timestamp);
    
    Ok(())
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
    
    // Stop zero publishing while we process this message
    SHOULD_PUBLISH_ZEROS.store(false, Ordering::SeqCst);
    
    // Process the data
    let output = {
        let mut state_guard = state.lock().unwrap();
        state_guard.update_last_receive(receive_time_ms);
        process_sensor_data(
            sensor_data,
            &mut state_guard,
            config.gap_fill_mode,
            config.gap_threshold_ms,
            config.pulses_per_liter,
            receive_time_ms,
        )
    };
    
    // Serialize just the timeseries array (not the full OutputMessage)
    let response = match serde_json::to_string(&output.timeseries) {
        Ok(json) => json,
        Err(e) => {
            error!("Failed to serialize output: {}", e);
            SHOULD_PUBLISH_ZEROS.store(true, Ordering::SeqCst);
            return Ok(());
        }
    };
    
    // Publish to MQTT
    client.publish(&config.publish_topic, QoS::AtLeastOnce, false, response.clone()).await?;
    
    info!(
        "Published {} entries ({} gap-filled with zeros) to {}",
        output.timeseries.len(),
        output.metadata.zero_entries_count,
        config.publish_topic
    );
    
    // Log the first and last entries for verification
    if let Some(first) = output.timeseries.first() {
        debug!("First entry: {} = {} L/min", first.timestamp, first.flow_rate_lpm);
    }
    if let Some(last) = output.timeseries.last() {
        debug!("Last entry: {} = {} L/min", last.timestamp, last.flow_rate_lpm);
    }
    
    debug!("Published data: {}", response);
    
    // Resume zero publishing
    SHOULD_PUBLISH_ZEROS.store(true, Ordering::SeqCst);
    
    Ok(())
}

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let config = Config::from_env();
    let state = create_state();
    
    info!("Starting Water Flow MQTT Service");
    info!("MQTT broker: {}:{}", config.mqtt_broker, config.mqtt_port);
    info!("Subscribe topic: {}", config.subscribe_topic);
    info!("Publish topic: {}", config.publish_topic);
    info!("Pulses per liter: {}", config.pulses_per_liter);
    info!("Gap fill mode: {:?}", config.gap_fill_mode);
    info!("Gap threshold: {} ms", config.gap_threshold_ms);
    
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
        info!("Will publish zero entries every 1 second when no data received");
        
        // Create a 1-second interval timer for zero publishing
        let mut zero_timer = interval(Duration::from_secs(1));
        // Allow the first tick to fire immediately
        zero_timer.tick().await;
        
        // Create a shared reference to client for the timer
        let client_for_timer = client.clone();
        let publish_topic = config.publish_topic.clone();
        
        // Main event loop
        loop {
            tokio::select! {
                // Handle MQTT events
                event = eventloop.poll() => {
                    match event {
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
                
                // Handle 1-second interval for zero publishing
                _ = zero_timer.tick() => {
                    if SHOULD_PUBLISH_ZEROS.load(Ordering::SeqCst) {
                        if let Err(e) = publish_zero_entry(&client_for_timer, &publish_topic, state.clone()).await {
                            error!("Failed to publish zero entry: {}", e);
                        }
                    }
                }
            }
        }
        
        warn!("Disconnected from MQTT broker. Reconnecting in 5 seconds...");
        sleep(Duration::from_secs(5)).await;
    }
}