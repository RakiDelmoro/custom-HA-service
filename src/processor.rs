use crate::config::GapFillMode;
use crate::models::{OutputMessage, SensorData, TimeseriesEntry};
use crate::state::{current_time_ms, ServiceState};
use chrono::{DateTime, FixedOffset, TimeZone};

// Hardcoded timezone offset for UTC+8 (Asia/Singapore, Hong Kong, etc.)
const TIMEZONE_OFFSET_HOURS: i32 = 8;
use log::{debug, info, warn};

/// Calculate L/min from pulses and seconds
/// Formula: (pulses / seconds) * (60 / pulses_per_liter)
pub fn calculate_l_per_min(pulses: u32, seconds: u64, pulses_per_liter: u32) -> f64 {
    if seconds == 0 || pulses_per_liter == 0 || pulses == 0 {
        return 0.0;
    }
    let pulses_per_second = pulses as f64 / seconds as f64;
    pulses_per_second * (60.0 / pulses_per_liter as f64)
}

/// Convert milliseconds since epoch to local timestamp string with month name
/// Format: "25 Feb 2026 09:11:21" (day Month year hour:minute:second)
/// Uses hardcoded UTC+8 offset for production
fn ms_to_timestamp(ms: u64) -> String {
    // Create timezone offset for UTC+8
    let offset =
        FixedOffset::east_opt(TIMEZONE_OFFSET_HOURS * 3600).unwrap_or(FixedOffset::east(0));
    let datetime: DateTime<FixedOffset> = offset.timestamp_millis_opt(ms as i64).unwrap();
    datetime.format("%d %b %Y %H:%M:%S").to_string()
}

/// Generate a single zero entry for the current time
pub fn generate_zero_entry() -> TimeseriesEntry {
    let now_ms = current_time_ms();
    TimeseriesEntry {
        timestamp: ms_to_timestamp(now_ms),
        flow_rate_lpm: 0.0,
    }
}

/// Process sensor data and generate output with timestamps
///
/// Returns an array of TimeseriesEntry with timestamps going backward from receive_time
pub fn process_sensor_data(
    data: SensorData,
    state: &mut ServiceState,
    _gap_fill_mode: GapFillMode,
    gap_threshold_ms: u64,
    pulses_per_liter: u32,
    receive_time_ms: u64,
) -> OutputMessage {
    let seconds = data.time_ms / 1000;

    debug!(
        "Processing: {} pulses over {} ms ({} seconds), received at {}",
        data.total_pulses,
        data.time_ms,
        seconds,
        ms_to_timestamp(receive_time_ms)
    );

    // Calculate L/min from current data
    let current_l_per_min = calculate_l_per_min(data.total_pulses, seconds, pulses_per_liter);

    // Calculate the start time of this sensor data
    // start_time = receive_time - sensor_duration
    let sensor_start_time_ms = receive_time_ms.saturating_sub(data.time_ms);

    let mut timeseries = Vec::new();
    let mut gap_filled = false;
    let mut zero_entries_count: u64 = 0;

    // Check if this is the first message
    if !state.is_initialized {
        info!(
            "First sensor message received at {}, initializing state",
            ms_to_timestamp(receive_time_ms)
        );
        state.is_initialized = true;
        state.last_l_per_min = current_l_per_min;
        state.last_sensor_time_ms = receive_time_ms;

        // For first message, just return entries for the sensor data duration
        for i in 0..seconds {
            let entry_time_ms = sensor_start_time_ms + (i * 1000);
            timeseries.push(TimeseriesEntry {
                timestamp: ms_to_timestamp(entry_time_ms),
                flow_rate_lpm: current_l_per_min,
            });
        }

        return OutputMessage::new(timeseries, false, 0);
    }

    // Calculate gap since last message
    let time_delta = receive_time_ms.saturating_sub(state.last_sensor_time_ms);

    // Check if there's a gap
    if time_delta > gap_threshold_ms {
        let gap_seconds = time_delta.saturating_sub(data.time_ms) / 1000;

        if gap_seconds > 0 {
            warn!(
                "Gap detected: {} seconds since last message (from {} to {})",
                gap_seconds,
                ms_to_timestamp(state.last_sensor_time_ms),
                ms_to_timestamp(receive_time_ms)
            );
            gap_filled = true;

            // Fill gap with zeros going back from sensor_start_time
            for i in (1..=gap_seconds).rev() {
                let gap_time_ms = sensor_start_time_ms.saturating_sub(i * 1000);
                timeseries.push(TimeseriesEntry {
                    timestamp: ms_to_timestamp(gap_time_ms),
                    flow_rate_lpm: 0.0,
                });
                zero_entries_count += 1;
            }
        }
    }

    // Add current data entries
    for i in 0..seconds {
        let entry_time_ms = sensor_start_time_ms + (i * 1000);
        timeseries.push(TimeseriesEntry {
            timestamp: ms_to_timestamp(entry_time_ms),
            flow_rate_lpm: current_l_per_min,
        });
    }

    // Update state
    state.last_l_per_min = current_l_per_min;
    state.last_sensor_time_ms = receive_time_ms;

    OutputMessage::new(timeseries, gap_filled, zero_entries_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_l_per_min() {
        // 433 pulses per liter, 10 pulses in 1 second
        // (10/1) * (60/433) = 1.386 L/min
        let result = calculate_l_per_min(10, 1, 433);
        assert!((result - 1.386).abs() < 0.001);
    }

    #[test]
    fn test_process_first_message() {
        let mut state = ServiceState::new();
        let data = SensorData {
            total_pulses: 8,
            time_ms: 1000,
        };

        let receive_time = 1705336230000u64; // Some arbitrary time
        let output = process_sensor_data(
            data,
            &mut state,
            GapFillMode::LastKnown,
            2000,
            433,
            receive_time,
        );

        assert_eq!(output.timeseries.len(), 1);
        assert!(!output.metadata.gap_filled);
        assert_eq!(output.metadata.zero_entries_count, 0);
        assert!(state.is_initialized);
    }

    #[test]
    fn test_process_with_gap() {
        let mut state = ServiceState::new();

        // First message
        let data1 = SensorData {
            total_pulses: 10,
            time_ms: 1000,
        };
        let receive_time1 = 1705336230000u64;
        process_sensor_data(
            data1,
            &mut state,
            GapFillMode::LastKnown,
            2000,
            433,
            receive_time1,
        );

        // Second message after 5 second gap, with 2 seconds of data
        let data2 = SensorData {
            total_pulses: 20,
            time_ms: 2000, // 2 seconds
        };
        let receive_time2 = receive_time1 + 5000; // 5 seconds later
        let output = process_sensor_data(
            data2,
            &mut state,
            GapFillMode::LastKnown,
            2000,
            433,
            receive_time2,
        );

        // Should have: 3 zero entries (gap) + 2 actual entries = 5 total
        assert_eq!(output.timeseries.len(), 5);
        assert!(output.metadata.gap_filled);
        assert_eq!(output.metadata.zero_entries_count, 3);

        // Check first 3 are zeros
        assert_eq!(output.timeseries[0].flow_rate_lpm, 0.0);
        assert_eq!(output.timeseries[1].flow_rate_lpm, 0.0);
        assert_eq!(output.timeseries[2].flow_rate_lpm, 0.0);

        // Last 2 have actual values
        assert!(output.timeseries[3].flow_rate_lpm > 0.0);
        assert!(output.timeseries[4].flow_rate_lpm > 0.0);
    }

    #[test]
    fn test_sensor_initialization() {
        // Scenario: Sensor powers on, first message after setup
        // total_pulses: 0 (no flow during setup), Time_ms: ~10000
        let mut state = ServiceState::new();
        let data = SensorData {
            total_pulses: 0,
            time_ms: 10000, // ~10 seconds during setup
        };

        let receive_time = 1705336230000u64;
        let output = process_sensor_data(
            data,
            &mut state,
            GapFillMode::LastKnown,
            2000,
            433,
            receive_time,
        );

        // Should return 0 L/min since no pulses detected during setup
        assert_eq!(output.timeseries.len(), 10); // 10 seconds
        assert!(output.timeseries.iter().all(|e| e.flow_rate_lpm == 0.0));
        assert!(!output.metadata.gap_filled);
        assert_eq!(output.metadata.zero_entries_count, 0);
        assert!(state.is_initialized);
        assert_eq!(state.last_l_per_min, 0.0);
    }

    #[test]
    fn test_generate_zero_entry() {
        let entry = generate_zero_entry();
        assert_eq!(entry.flow_rate_lpm, 0.0);
        // Should have a valid timestamp format with month name (e.g., "25 Feb 2026 09:11:21")
        assert!(entry.timestamp.contains(' '));
        // Should contain a 3-letter month abbreviation
        let months = [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ];
        assert!(months.iter().any(|&m| entry.timestamp.contains(m)));
    }
}
