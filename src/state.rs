use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

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
    /// Timestamp of last data received (ms since epoch)
    pub last_data_receive_time_ms: Option<u64>,
    /// Set of all published timestamps to prevent duplicates
    pub published_timestamps: HashSet<u64>,
}

impl ServiceState {
    pub fn new() -> Self {
        ServiceState {
            last_l_per_min: 0.0,
            last_sensor_time_ms: 0,
            is_initialized: false,
            last_data_receive_time_ms: None,
            published_timestamps: HashSet::new(),
        }
    }

    /// Update the last data receive timestamp
    pub fn update_last_receive(&mut self, timestamp_ms: u64) {
        self.last_data_receive_time_ms = Some(timestamp_ms);
    }

    /// Check if timestamp was already published
    pub fn is_timestamp_published(&self, timestamp: u64) -> bool {
        self.published_timestamps.contains(&timestamp)
    }

    /// Mark timestamp as published
    pub fn mark_timestamp_published(&mut self, timestamp: u64) {
        self.published_timestamps.insert(timestamp);
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
