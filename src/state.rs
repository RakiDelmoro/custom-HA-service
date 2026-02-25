use std::sync::{Arc, Mutex};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Shared state for the service
pub type SharedState = Arc<Mutex<ServiceState>>;

#[derive(Debug)]
pub struct ServiceState {
    /// Last calculated L/min value
    pub last_l_per_min: f64,
    /// Timestamp of last sensor message (ms since epoch)
    pub last_sensor_time_ms: u64,
    /// Whether we've received the first message
    pub is_initialized: bool,
    /// Time when service started
    pub service_start_time: Instant,
    /// Timestamp of last data received (ms since epoch)
    pub last_data_receive_time_ms: Option<u64>,
    /// Last zero publish timestamp string (to prevent duplicates)
    pub last_zero_publish_timestamp: Option<String>,
}

impl ServiceState {
    pub fn new() -> Self {
        ServiceState {
            last_l_per_min: 0.0,
            last_sensor_time_ms: 0,
            is_initialized: false,
            service_start_time: Instant::now(),
            last_data_receive_time_ms: None,
            last_zero_publish_timestamp: None,
        }
    }

    /// Update the last data receive timestamp
    pub fn update_last_receive(&mut self, timestamp_ms: u64) {
        self.last_data_receive_time_ms = Some(timestamp_ms);
    }

    /// Check if we should publish a zero entry (prevents duplicate timestamps)
    pub fn should_publish_zero(&mut self, timestamp: &str) -> bool {
        if let Some(ref last) = self.last_zero_publish_timestamp {
            if last == timestamp {
                // Same timestamp as last publish, skip it
                return false;
            }
        }
        // Update and allow publish
        self.last_zero_publish_timestamp = Some(timestamp.to_string());
        true
    }
}

/// Get current time in milliseconds since epoch
pub fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Create shared state
pub fn create_state() -> SharedState {
    Arc::new(Mutex::new(ServiceState::new()))
}
