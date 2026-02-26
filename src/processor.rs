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
pub fn process_sensor_data(
    data: SensorData,
    state: &mut ServiceState,
    pulses_per_liter: u32,
    receive_time_ms: u64,
) -> Vec<TimeseriesEntry> {
    let seconds = data.time_ms / 1000;

    debug!(
        "Processing: {} pulses over {} ms ({} seconds), received at epoch {}",
        data.total_pulses, data.time_ms, seconds, receive_time_ms
    );

    // Calculate L/min from current data
    let current_l_per_min = calculate_l_per_min(data.total_pulses, seconds, pulses_per_liter);

    // Calculate the start time of this sensor data
    // start_time = receive_time - sensor_duration
    let sensor_start_time_ms = receive_time_ms.saturating_sub(data.time_ms);

    let mut timeseries = Vec::new();

    // Check if this is the first message
    if !state.is_initialized {
        info!(
            "First sensor message received at epoch {}, initializing state",
            receive_time_ms
        );
        state.is_initialized = true;
        state.last_l_per_min = current_l_per_min;
        state.last_sensor_time_ms = receive_time_ms;

        // For first message, just return entries for the sensor data duration
        for i in 0..seconds {
            let entry_time_ms = sensor_start_time_ms + (i * 1000);
            timeseries.push(TimeseriesEntry {
                timestamp: entry_time_ms,
                flow_rate_lpm: current_l_per_min,
            });
        }

        return timeseries;
    }

    // Calculate gap since last message and always fill
    // Round up to ensure no gaps are missed (add 999ms before dividing)
    let time_delta = receive_time_ms.saturating_sub(state.last_sensor_time_ms);
    let gap_seconds = time_delta.saturating_sub(data.time_ms).saturating_add(999) / 1000;

    if gap_seconds > 0 {
        warn!(
            "Gap detected: {} seconds since last message (from {} to {})",
            gap_seconds, state.last_sensor_time_ms, receive_time_ms
        );

        // Fill gap with zeros going back from sensor_start_time
        for i in (1..=gap_seconds).rev() {
            let gap_time_ms = sensor_start_time_ms.saturating_sub(i * 1000);
            timeseries.push(TimeseriesEntry {
                timestamp: gap_time_ms,
                flow_rate_lpm: 0.0,
            });
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
    state.last_sensor_time_ms = receive_time_ms;

    timeseries
}
