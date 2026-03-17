//! Data models for FlowPulse MQTT service
//!
//! This module defines the core data structures used throughout the service:
//!
//! - **SensorData**: Input data from pulse sensors
//! - **TimeseriesEntry**: Output data with calculated flow rates
//!
//! # Serialization
//!
//! Both structs use serde for JSON serialization/deserialization.
//! Field names in JSON may differ from Rust field names (e.g., `Time_ms` vs `time_ms`).

use serde::{Deserialize, Serialize};

/// Input data from pulse sensors
///
/// This structure represents the raw data received from ESP32 or similar
/// pulse-counting devices. The JSON format uses specific field names
/// for compatibility with existing sensor firmware.
///
/// # JSON Format
///
/// ```json
/// {
///   "total_pulses": 433,
///   "Time_ms": 10000
/// }
/// ```
///
/// # Fields
///
/// - `total_pulses`: Total number of pulses counted in the measurement period
/// - `time_ms`: Duration of the measurement in milliseconds
///
/// # Example
///
/// ```rust
/// use custom_ha_service::models::SensorData;
/// use serde_json;
///
/// let json = r#"{"total_pulses": 433, "Time_ms": 10000}"#;
/// let data: SensorData = serde_json::from_str(json).unwrap();
/// assert_eq!(data.total_pulses, 433);
/// assert_eq!(data.time_ms, 10000);
/// ```
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct SensorData {
    /// Total number of pulses counted
    #[serde(rename = "total_pulses")]
    pub total_pulses: u32,
    /// Duration of measurement in milliseconds
    #[serde(rename = "Time_ms")]
    pub time_ms: u64,
}

impl SensorData {
    /// Creates a new SensorData instance
    ///
    /// This is primarily useful for testing.
    ///
    /// # Example
    ///
    /// ```rust
    /// use custom_ha_service::models::SensorData;
    ///
    /// let data = SensorData::new(433, 1000);
    /// assert_eq!(data.total_pulses, 433);
    /// assert_eq!(data.time_ms, 1000);
    /// ```
    pub fn new(total_pulses: u32, time_ms: u64) -> Self {
        SensorData {
            total_pulses,
            time_ms,
        }
    }

    /// Returns the duration in seconds (rounded to nearest second)
    ///
    /// Uses the same rounding logic as the processor:
    /// - Values < 500ms round to 0 seconds
    /// - Values >= 500ms round to 1 second
    pub fn duration_secs(&self) -> u64 {
        (self.time_ms + 500) / 1000
    }

    /// Calculates flow rate in liters per minute
    ///
    /// Formula: (pulses / seconds) * (60 / pulses_per_liter)
    ///
    /// # Example
    ///
    /// ```rust
    /// use custom_ha_service::models::SensorData;
    ///
    /// let data = SensorData::new(433, 1000); // 1 second, 433 pulses
    /// let lpm = data.flow_rate_lpm(433); // 433 pulses per liter
    /// assert!((lpm - 60.0).abs() < 0.01); // 60 L/min
    /// ```
    pub fn flow_rate_lpm(&self, pulses_per_liter: u32) -> f64 {
        let seconds = self.duration_secs();
        if seconds == 0 || pulses_per_liter == 0 || self.total_pulses == 0 {
            return 0.0;
        }
        let pulses_per_second = self.total_pulses as f64 / seconds as f64;
        pulses_per_second * (60.0 / pulses_per_liter as f64)
    }
}

/// Single timeseries entry with timestamp and flow rate
///
/// This structure represents a single data point in the output timeseries.
/// Each entry has a Unix timestamp (in seconds) and a calculated flow rate.
///
/// # JSON Format
///
/// ```json
/// {
///   "timestamp": 1705336230,
///   "flow_rate_lpm": 60.0
/// }
/// ```
///
/// # Fields
///
/// - `timestamp`: Unix timestamp in seconds (UTC)
/// - `flow_rate_lpm`: Flow rate in liters per minute
///
/// # Example
///
/// ```rust
/// use custom_ha_service::models::TimeseriesEntry;
///
/// let entry = TimeseriesEntry::new(1705336230, 60.0);
/// let json = serde_json::to_string(&entry).unwrap();
/// assert!(json.contains("timestamp"));
/// assert!(json.contains("flow_rate_lpm"));
/// ```
#[derive(Debug, Serialize, Clone, PartialEq)]
pub struct TimeseriesEntry {
    /// Unix timestamp in seconds (UTC)
    #[serde(rename = "timestamp")]
    pub timestamp: u64,
    /// Flow rate in liters per minute
    #[serde(rename = "flow_rate_lpm")]
    pub flow_rate_lpm: f64,
}

impl TimeseriesEntry {
    /// Creates a new TimeseriesEntry
    ///
    /// # Example
    ///
    /// ```rust
    /// use custom_ha_service::models::TimeseriesEntry;
    ///
    /// let entry = TimeseriesEntry::new(1705336230, 60.0);
    /// assert_eq!(entry.timestamp, 1705336230);
    /// assert!((entry.flow_rate_lpm - 60.0).abs() < 0.01);
    /// ```
    pub fn new(timestamp: u64, flow_rate_lpm: f64) -> Self {
        TimeseriesEntry {
            timestamp,
            flow_rate_lpm,
        }
    }

    /// Creates a zero-flow entry (for gap filling)
    ///
    /// # Example
    ///
    /// ```rust
    /// use custom_ha_service::models::TimeseriesEntry;
    ///
    /// let entry = TimeseriesEntry::zero(1705336230);
    /// assert_eq!(entry.timestamp, 1705336230);
    /// assert_eq!(entry.flow_rate_lpm, 0.0);
    /// ```
    pub fn zero(timestamp: u64) -> Self {
        TimeseriesEntry {
            timestamp,
            flow_rate_lpm: 0.0,
        }
    }

    /// Checks if this entry represents zero flow
    pub fn is_zero(&self) -> bool {
        self.flow_rate_lpm == 0.0
    }
}

/// Batch of timeseries entries for efficient processing
#[allow(dead_code)]
pub type TimeseriesBatch = Vec<TimeseriesEntry>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_data_deserialization() {
        let json = r#"{"total_pulses": 433, "Time_ms": 10000}"#;
        let data: SensorData = serde_json::from_str(json).unwrap();
        assert_eq!(data.total_pulses, 433);
        assert_eq!(data.time_ms, 10000);
    }

    #[test]
    fn test_sensor_data_duration_secs() {
        // Values < 500ms round to 0
        assert_eq!(SensorData::new(100, 499).duration_secs(), 0);
        // Values >= 500ms round to 1
        assert_eq!(SensorData::new(100, 500).duration_secs(), 1);
        // Normal rounding
        assert_eq!(SensorData::new(100, 1500).duration_secs(), 2);
    }

    #[test]
    fn test_sensor_data_flow_rate_calculation() {
        // 433 pulses in 1 second at 433 pulses/liter = 60 L/min
        let data = SensorData::new(433, 1000);
        let lpm = data.flow_rate_lpm(433);
        assert!((lpm - 60.0).abs() < 0.01);
    }

    #[test]
    fn test_sensor_data_zero_flow() {
        let data = SensorData::new(0, 1000);
        assert_eq!(data.flow_rate_lpm(433), 0.0);
    }

    #[test]
    fn test_timeseries_entry_serialization() {
        let entry = TimeseriesEntry::new(1705336230, 60.0);
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("flow_rate_lpm"));
        assert!(json.contains("1705336230"));
    }

    #[test]
    fn test_timeseries_entry_zero() {
        let entry = TimeseriesEntry::zero(1705336230);
        assert_eq!(entry.timestamp, 1705336230);
        assert_eq!(entry.flow_rate_lpm, 0.0);
        assert!(entry.is_zero());
    }

    #[test]
    fn test_sensor_data_equality() {
        let data1 = SensorData::new(433, 1000);
        let data2 = SensorData::new(433, 1000);
        let data3 = SensorData::new(434, 1000);
        assert_eq!(data1, data2);
        assert_ne!(data1, data3);
    }

    #[test]
    fn test_timeseries_entry_equality() {
        let entry1 = TimeseriesEntry::new(100, 60.0);
        let entry2 = TimeseriesEntry::new(100, 60.0);
        let entry3 = TimeseriesEntry::new(100, 61.0);
        assert_eq!(entry1, entry2);
        assert_ne!(entry1, entry3);
    }
}
