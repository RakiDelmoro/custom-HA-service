use crate::models::{SensorData, TimeseriesEntry};
use crate::state::ServiceState;
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

/// Process sensor data and generate output with timestamps
///
/// Returns a Vec of TimeseriesEntry with timestamps going backward from receive_time
/// Automatically fills any gaps with zero entries to ensure no data is missed
///
/// Gap calculation:
/// - gap_ms = receive_time_ms - last_published_timestamp - time_ms
/// - Gap timestamps are filled with 0 L/min
/// - Actual data timestamps get calculated L/min from sensor data
///
/// Handles variable sensor timing by rounding time_ms to nearest second and
/// ensuring timestamps never overlap with previously published data.
pub fn process_sensor_data(
    data: SensorData,
    state: &mut ServiceState,
    pulses_per_liter: u32,
    receive_time_ms: u64,
) -> Vec<TimeseriesEntry> {
    // Round sensor duration to nearest second to handle variable ESP32 timing
    // ESP32 sends time_ms like 1000, 1030, 1425, 15060 - round to nearest 1000ms
    let rounded_time_ms = ((data.time_ms + 500) / 1000) * 1000;
    let seconds = rounded_time_ms / 1000;

    debug!(
        "Processing: {} pulses over {} ms (rounded to {} ms, {} seconds), received at epoch {}",
        data.total_pulses, data.time_ms, rounded_time_ms, seconds, receive_time_ms
    );

    // Calculate L/min from current data (use actual pulses, rounded seconds)
    let current_l_per_min = calculate_l_per_min(data.total_pulses, seconds, pulses_per_liter);

    // Calculate the start time of this sensor data
    // start_time = receive_time - sensor_duration (using rounded time)
    let raw_sensor_start_time_ms = receive_time_ms.saturating_sub(rounded_time_ms);

    // For first message, just use the calculated start time
    // For subsequent messages, ensure we don't overlap with last published timestamp
    let sensor_start_time_ms = if !state.is_initialized {
        raw_sensor_start_time_ms
    } else {
        // Ensure sensor_start_time is always after last_published_timestamp
        // This prevents duplicate/overlapping timestamps when ESP32 timing varies
        let min_start_time = state.last_published_timestamp + 1000;
        std::cmp::max(raw_sensor_start_time_ms, min_start_time)
    };

    let mut timeseries = Vec::new();

    // Check if this is the first message
    if !state.is_initialized {
        info!(
            "First sensor message received at epoch {}, initializing state",
            receive_time_ms
        );
        state.is_initialized = true;
        state.last_l_per_min = current_l_per_min;

        // For first message, return entries for the sensor data duration
        for i in 0..seconds {
            let entry_time_ms = sensor_start_time_ms + (i * 1000);
            timeseries.push(TimeseriesEntry {
                timestamp: entry_time_ms,
                flow_rate_lpm: current_l_per_min,
            });
        }

        // Update last_published_timestamp to the end of this data
        if seconds > 0 {
            state.last_published_timestamp = sensor_start_time_ms + ((seconds - 1) * 1000);
        }

        return timeseries;
    }

    // Calculate gap since last published timestamp
    // Gap = receive_time_ms - last_published_timestamp - rounded_time_ms
    // Use rounded_time_ms for consistent gap calculation despite ESP32 timing variations
    let total_time_since_last = receive_time_ms.saturating_sub(state.last_published_timestamp);
    let gap_ms = total_time_since_last.saturating_sub(rounded_time_ms);
    let gap_seconds = gap_ms / 1000;

    if gap_seconds > 0 {
        warn!(
            "Gap detected: {} seconds (from {} to {})",
            gap_seconds, state.last_published_timestamp, sensor_start_time_ms
        );

        // Fill gap with zeros
        // Gap timestamps go from last_published_timestamp + 1000ms to sensor_start_time_ms - 1000ms
        let gap_start_ms = state.last_published_timestamp + 1000;
        let gap_end_ms = sensor_start_time_ms;

        let mut current_gap_ms = gap_start_ms;
        while current_gap_ms < gap_end_ms {
            timeseries.push(TimeseriesEntry {
                timestamp: current_gap_ms,
                flow_rate_lpm: 0.0,
            });
            current_gap_ms += 1000;
        }
    }

    // Add current data entries
    for i in 0..seconds {
        let entry_time_ms = sensor_start_time_ms + (i * 1000);
        timeseries.push(TimeseriesEntry {
            timestamp: entry_time_ms,
            flow_rate_lpm: current_l_per_min,
        });
    }

    // Update state
    state.last_l_per_min = current_l_per_min;
    // Update last_published_timestamp to the end of current data
    if seconds > 0 {
        state.last_published_timestamp = sensor_start_time_ms + ((seconds - 1) * 1000);
    }

    timeseries
}
