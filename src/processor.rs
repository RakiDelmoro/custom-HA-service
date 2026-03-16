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
/// - gap_seconds = receive_time_sec - last_published_timestamp - seconds
/// - Gap timestamps are filled with 0 L/min
/// - Actual data timestamps get calculated L/min from sensor data
///
/// Handles variable sensor timing by rounding time_ms to nearest second and
/// ensuring timestamps never overlap with previously published data.
pub fn process_sensor_data(
    data: SensorData,
    state: &mut ServiceState,
    pulses_per_liter: u32,
    receive_time_sec: u64,
) -> Vec<TimeseriesEntry> {
    // Convert ms to seconds, rounding to nearest second
    let seconds = ((data.time_ms + 500) / 1000) as u64;

    debug!(
        "Processing: {} pulses over {} ms (rounded to {} seconds), received at epoch {}",
        data.total_pulses, data.time_ms, seconds, receive_time_sec
    );

    // Calculate L/min from current data
    let current_l_per_min = calculate_l_per_min(data.total_pulses, seconds, pulses_per_liter);

    // Calculate the start time of this sensor data (in seconds)
    let raw_sensor_start_time_sec = receive_time_sec.saturating_sub(seconds);

    // For first message, just use the calculated start time
    // For subsequent messages, ensure we don't overlap with last published timestamp
    let sensor_start_time_sec = if !state.is_initialized {
        raw_sensor_start_time_sec
    } else {
        // Ensure sensor_start_time is always after last_published_timestamp
        let min_start_time = state.last_published_timestamp + 1;
        std::cmp::max(raw_sensor_start_time_sec, min_start_time)
    };

    let mut timeseries = Vec::new();

    // Check if this is the first message
    if !state.is_initialized {
        info!(
            "First sensor message received at epoch {}, initializing state",
            receive_time_sec
        );
        state.is_initialized = true;
        state.last_l_per_min = current_l_per_min;

        // For first message, return entries for the sensor data duration
        for i in 0..seconds {
            let entry_time_sec = sensor_start_time_sec + i;
            timeseries.push(TimeseriesEntry {
                timestamp: entry_time_sec,
                flow_rate_lpm: current_l_per_min,
            });
        }

        // Update last_published_timestamp to the end of this data
        if seconds > 0 {
            state.last_published_timestamp = sensor_start_time_sec + (seconds - 1);
        }

        return timeseries;
    }

    // Calculate gap since last published timestamp (in seconds)
    let total_time_since_last = receive_time_sec.saturating_sub(state.last_published_timestamp);
    let gap_seconds = total_time_since_last.saturating_sub(seconds);

    if gap_seconds > 0 {
        warn!(
            "Gap detected: {} seconds (from {} to {})",
            gap_seconds, state.last_published_timestamp, sensor_start_time_sec
        );

        // Fill gap with zeros
        let gap_start_sec = state.last_published_timestamp + 1;
        let gap_end_sec = sensor_start_time_sec;

        let mut current_gap_sec = gap_start_sec;
        while current_gap_sec < gap_end_sec {
            timeseries.push(TimeseriesEntry {
                timestamp: current_gap_sec,
                flow_rate_lpm: 0.0,
            });
            current_gap_sec += 1;
        }
    }

    // Add current data entries
    for i in 0..seconds {
        let entry_time_sec = sensor_start_time_sec + i;
        timeseries.push(TimeseriesEntry {
            timestamp: entry_time_sec,
            flow_rate_lpm: current_l_per_min,
        });
    }

    // Update state
    state.last_l_per_min = current_l_per_min;
    if seconds > 0 {
        state.last_published_timestamp = sensor_start_time_sec + (seconds - 1);
    }

    timeseries
}
