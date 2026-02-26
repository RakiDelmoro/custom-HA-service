use serde::{Deserialize, Serialize};

/// Input data from flow sensor
#[derive(Debug, Deserialize)]
pub struct SensorData {
    #[serde(rename = "total_pulses")]
    pub total_pulses: u32,
    #[serde(rename = "Time_ms")]
    pub time_ms: u64,
}

/// Single timeseries entry with timestamp and flow rate
#[derive(Debug, Serialize, Clone)]
pub struct TimeseriesEntry {
    /// Unix timestamp in milliseconds
    pub timestamp: u64,
    /// Flow rate in liters per minute
    #[serde(rename = "flow_rate_lpm")]
    pub flow_rate_lpm: f64,
}
