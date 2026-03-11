use custom_ha_service::in_memory_mock::InMemoryMqttMock;
use custom_ha_service::models::SensorData;
use custom_ha_service::processor::process_sensor_data;
use custom_ha_service::state::ServiceState;
use std::time::{SystemTime, UNIX_EPOCH};

/// Integration tests using in-memory MQTT mock
/// These tests don't require an external MQTT broker
fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[test]
fn test_basic_flow_calculation() {
    let mut state = ServiceState::new();
    let data = SensorData {
        total_pulses: 433,
        time_ms: 1000,
    };

    let receive_time: u64 = current_time_ms();
    let entries = process_sensor_data(data, &mut state, 433, receive_time);

    assert_eq!(entries.len(), 1, "Should have 1 entry for 1 second of data");

    let entry = &entries[0];
    // 433 pulses / 1 second at 433 pulses/liter = 60 L/min
    let expected: f64 = 60.0;
    assert!(
        (entry.flow_rate_lpm - expected).abs() < 0.1,
        "Expected ~60 L/min, got {}",
        entry.flow_rate_lpm
    );
}

#[test]
fn test_zero_flow() {
    let mut state = ServiceState::new();
    let data = SensorData {
        total_pulses: 0,
        time_ms: 10000,
    };

    let receive_time: u64 = current_time_ms();
    let entries = process_sensor_data(data, &mut state, 433, receive_time);

    assert_eq!(entries.len(), 10, "Should have 10 entries for 10 seconds");

    for entry in &entries {
        assert_eq!(
            entry.flow_rate_lpm, 0.0,
            "Zero pulses should result in 0.0 L/min"
        );
    }
}

#[test]
fn test_gap_detection() {
    let mut state = ServiceState::new();

    // First message
    let data1 = SensorData {
        total_pulses: 433,
        time_ms: 1000,
    };
    let receive_time1: u64 = 1705336230000u64;
    let entries1 = process_sensor_data(data1, &mut state, 433, receive_time1);

    assert_eq!(entries1.len(), 1, "First message should have 1 entry");
    assert!(entries1[0].flow_rate_lpm > 0.0, "Should have flow rate");

    // Second message after 5 second gap
    let data2 = SensorData {
        total_pulses: 866,
        time_ms: 2000, // 2 seconds of data
    };
    let receive_time2: u64 = receive_time1 + 5000; // 5 seconds later
    let entries2 = process_sensor_data(data2, &mut state, 433, receive_time2);

    // Should have: 3 gap entries + 2 data entries = 5 total
    assert_eq!(entries2.len(), 5, "Should have 5 entries (3 gap + 2 data)");

    // First 3 should be zeros (gap)
    assert_eq!(entries2[0].flow_rate_lpm, 0.0);
    assert_eq!(entries2[1].flow_rate_lpm, 0.0);
    assert_eq!(entries2[2].flow_rate_lpm, 0.0);

    // Last 2 should have actual flow
    assert!(entries2[3].flow_rate_lpm > 0.0);
    assert!(entries2[4].flow_rate_lpm > 0.0);
}

#[test]
fn test_multiple_data_points() {
    let mut state = ServiceState::new();

    let data = SensorData {
        total_pulses: 2165, // 5 seconds worth at 433 pulses/liter
        time_ms: 5000,
    };

    let receive_time: u64 = current_time_ms();
    let entries = process_sensor_data(data, &mut state, 433, receive_time);

    assert_eq!(entries.len(), 5, "Should have 5 entries for 5 seconds");

    // All entries should have the same flow rate
    let expected_rate: f64 = 60.0; // 2165 pulses / 5 seconds = 433 pulses/second = 60 L/min
    for (i, entry) in entries.iter().enumerate() {
        assert!(
            (entry.flow_rate_lpm - expected_rate).abs() < 0.1,
            "Entry {}: Expected ~{} L/min, got {}",
            i,
            expected_rate,
            entry.flow_rate_lpm
        );
    }
}

#[test]
fn test_in_memory_mqtt_mock() {
    let mock = InMemoryMqttMock::new();

    // Subscribe to input and output topics
    let mut input_rx = mock.subscribe("custom-service/in");
    let _output_rx = mock.subscribe("flowpulse/out");

    // Simulate sensor publishing
    mock.publish(
        "custom-service/in",
        r#"{"total_pulses": 433, "Time_ms": 1000}"#,
    )
    .unwrap();

    // Verify message was published
    let messages = mock.get_published_messages();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].0, "custom-service/in");

    // Verify subscriber received it
    let received = input_rx.try_recv();
    assert!(received.is_ok());
    assert_eq!(
        received.unwrap(),
        r#"{"total_pulses": 433, "Time_ms": 1000}"#
    );
}

#[test]
fn test_duplicate_prevention() {
    let mut state = ServiceState::new();
    state.is_initialized = true;

    // Mark some timestamps as already published
    let test_timestamp: u64 = 1705336230000u64;
    state.mark_timestamp_published(test_timestamp);

    assert!(
        state.is_timestamp_published(test_timestamp),
        "Should detect published timestamp"
    );
    assert!(
        !state.is_timestamp_published(test_timestamp + 1000),
        "Should not detect unpublished"
    );
}

#[test]
fn test_gap_fill_with_last_published_timestamp() {
    let mut state = ServiceState::new();

    // First message: 2 seconds of data at receive_time = 10000ms
    // Sensor data covers: 8000ms to 10000ms
    let data1 = SensorData {
        total_pulses: 866, // 2 seconds worth
        time_ms: 2000,
    };
    let receive_time1: u64 = 10000;
    let entries1 = process_sensor_data(data1, &mut state, 433, receive_time1);

    // Should have 2 entries: 8000ms and 9000ms
    assert_eq!(entries1.len(), 2);
    assert_eq!(entries1[0].timestamp, 8000);
    assert_eq!(entries1[1].timestamp, 9000);
    assert!(entries1[0].flow_rate_lpm > 0.0);
    assert!(entries1[1].flow_rate_lpm > 0.0);

    // Verify last_published_timestamp updated
    assert_eq!(state.last_published_timestamp, 9000);

    // Second message: 3 seconds of data, received 5 seconds after last published
    // Receive time = 9000 + 5000 = 14000ms
    // Sensor data covers: 14000 - 3000 = 11000ms to 14000ms
    // Gap should be from 10000ms to 11000ms (1 second)
    let data2 = SensorData {
        total_pulses: 1299, // 3 seconds worth
        time_ms: 3000,
    };
    let receive_time2: u64 = 15000; // 15000 - 9000 = 6000ms after last published
    let entries2 = process_sensor_data(data2, &mut state, 433, receive_time2);

    // Should have: 1 gap entry (10000ms) + 3 data entries (12000, 13000, 14000ms)
    // Note: Gap is 15000 - 9000 - 3000 = 3000ms = 3 seconds
    // But sensor_start = 15000 - 3000 = 12000ms
    // So gap is from 10000 to 12000 = 2 seconds (10000, 11000)
    assert_eq!(entries2.len(), 5, "Should have 5 entries (2 gap + 3 data)");

    // First 2 should be gap zeros
    assert_eq!(entries2[0].timestamp, 10000);
    assert_eq!(entries2[0].flow_rate_lpm, 0.0);
    assert_eq!(entries2[1].timestamp, 11000);
    assert_eq!(entries2[1].flow_rate_lpm, 0.0);

    // Last 3 should have actual flow
    assert_eq!(entries2[2].timestamp, 12000);
    assert!(entries2[2].flow_rate_lpm > 0.0);
    assert_eq!(entries2[3].timestamp, 13000);
    assert!(entries2[3].flow_rate_lpm > 0.0);
    assert_eq!(entries2[4].timestamp, 14000);
    assert!(entries2[4].flow_rate_lpm > 0.0);

    // Verify last_published_timestamp updated to end of this data
    assert_eq!(state.last_published_timestamp, 14000);
}

#[test]
fn test_variable_esp32_timing() {
    let mut state = ServiceState::new();

    // First message: ESP32 sends 1030ms (rounded to 1000ms)
    let data1 = SensorData {
        total_pulses: 433,
        time_ms: 1030, // Variable timing from ESP32
    };
    let receive_time1: u64 = 10000;
    let entries1 = process_sensor_data(data1, &mut state, 433, receive_time1);

    // Should have 1 entry at timestamp 9000 (10000 - 1000 rounded)
    assert_eq!(
        entries1.len(),
        1,
        "Should have 1 entry for 1030ms rounded to 1000ms"
    );
    assert_eq!(entries1[0].timestamp, 9000);
    assert!(entries1[0].flow_rate_lpm > 0.0);
    assert_eq!(state.last_published_timestamp, 9000);

    // Second message: ESP32 sends 1425ms (rounded to 1000ms)
    // Received 1000ms after first message
    let data2 = SensorData {
        total_pulses: 433,
        time_ms: 1425, // Variable timing
    };
    let receive_time2: u64 = 11000;
    let entries2 = process_sensor_data(data2, &mut state, 433, receive_time2);

    // Should have 1 entry at timestamp 10000 (11000 - 1000 rounded)
    // No gap because: 11000 - 9000 - 1000 = 1000ms gap, but rounded to 0 seconds
    assert_eq!(entries2.len(), 1, "Should handle variable 1425ms timing");
    assert_eq!(entries2[0].timestamp, 10000);
    assert!(entries2[0].flow_rate_lpm > 0.0);
    assert_eq!(state.last_published_timestamp, 10000);

    // Third message: ESP32 sends 15060ms (15 seconds rounded)
    // Received after a delay
    let data3 = SensorData {
        total_pulses: 6495, // 15 seconds worth at 433 pulses/liter
        time_ms: 15060,     // Variable timing - 15 seconds
    };
    let receive_time3: u64 = 26000; // 15 seconds after receive_time2
    let entries3 = process_sensor_data(data3, &mut state, 433, receive_time3);

    // Gap calculation: 26000 - 10000 - 15000 = 1000ms = 1 second gap
    // However, with the min_start_time protection, gap is exactly at boundary
    // So: 0 gap entries + 15 data entries = 15 total (gap of exactly 1 second is edge case)
    assert_eq!(entries3.len(), 15, "Should handle 15-second burst");

    // First entry should be at 11000ms (data starts immediately after gap boundary)
    assert_eq!(entries3[0].timestamp, 11000);
    assert!(entries3[0].flow_rate_lpm > 0.0);

    // Last entry should be at 25000ms (26000 - 1000 rounded from 15060)
    assert_eq!(entries3[14].timestamp, 25000);
    assert!(entries3[14].flow_rate_lpm > 0.0);

    // Verify no duplicate timestamps (all should be unique and increasing)
    for i in 1..entries3.len() {
        assert!(
            entries3[i].timestamp > entries3[i - 1].timestamp,
            "Timestamps should always increase, no duplicates"
        );
    }

    // Verify last_published_timestamp updated correctly
    assert_eq!(state.last_published_timestamp, 25000);
}
