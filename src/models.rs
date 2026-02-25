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
    /// ISO 8601 timestamp (YYYY-MM-DDTHH:MM:SS format, local timezone)
    pub timestamp: String,
    /// Flow rate in liters per minute
    #[serde(rename = "flow_rate_lpm")]
    pub flow_rate_lpm: f64,
}

/// Output message with flow rate data
#[derive(Debug, Serialize)]
pub struct OutputMessage {
    /// Timeseries array with timestamps and flow rates
    pub timeseries: Vec<TimeseriesEntry>,
    pub metadata: Metadata,
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    /// Duration of the measurement in seconds
    pub duration_seconds: u64,
    /// Average flow rate in liters per minute
    pub average_lpm: f64,
    /// Whether any gaps were filled in the data
    pub gap_filled: bool,
    /// Number of entries with zero flow (gap fill)
    pub zero_entries_count: u64,
}

impl OutputMessage {
    pub fn new(
        timeseries: Vec<TimeseriesEntry>,
        gap_filled: bool,
        zero_entries_count: u64,
    ) -> Self {
        let duration_seconds = timeseries.len() as u64;
        let average_lpm = if timeseries.is_empty() {
            0.0
        } else {
            timeseries.iter().map(|e| e.flow_rate_lpm).sum::<f64>() / timeseries.len() as f64
        };

        OutputMessage {
            timeseries,
            metadata: Metadata {
                duration_seconds,
                average_lpm,
                gap_filled,
                zero_entries_count,
            },
        }
    }
}

/// Creates a zero-value entry for the current time
pub fn create_zero_entry(timestamp_str: String) -> TimeseriesEntry {
    TimeseriesEntry {
        timestamp: timestamp_str,
        flow_rate_lpm: 0.0,
    }
}
