use custom_ha_service::models::SensorData;
use custom_ha_service::processor::process_sensor_data;
use custom_ha_service::state::ServiceState;
use custom_ha_service::test_utils::InMemoryMqttMock;
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
fn test_gap_detection() {
    let mut state = ServiceState::new();

    // First message - use seconds-based timestamps (Unix timestamps in seconds)
    let data1 = SensorData {
        total_pulses: 433,
        time_ms: 1000, // 1 second of data (will be rounded)
    };
    let receive_time1: u64 = 1000; // Unix timestamp in seconds
    let entries1 = process_sensor_data(data1, &mut state, 433, receive_time1);

    assert_eq!(entries1.len(), 1, "First message should have 1 entry");
    assert!(entries1[0].flow_rate_lpm > 0.0, "Should have flow rate");
    assert_eq!(entries1[0].timestamp, 999, "Should be at timestamp 999");

    // Second message after 5 second gap
    let data2 = SensorData {
        total_pulses: 866,
        time_ms: 2000, // 2 seconds of data (will be rounded)
    };
    let receive_time2: u64 = receive_time1 + 8; // 8 seconds later (includes 5s gap + 2s data + 1s margin)
    let entries2 = process_sensor_data(data2, &mut state, 433, receive_time2);

    // Should have: gap entries + data entries
    // Gap: from 1000 to 1006 = 6 seconds
    // Data: 2 seconds = 2 entries
    // Total should be reasonable
    assert!(
        entries2.len() >= 2,
        "Should have at least 2 data entries, got {}",
        entries2.len()
    );

    // Verify timestamps are monotonically increasing
    for i in 1..entries2.len() {
        assert!(
            entries2[i].timestamp > entries2[i - 1].timestamp,
            "Timestamps should increase: {} -> {}",
            entries2[i - 1].timestamp,
            entries2[i].timestamp
        );
    }
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

    // First message: 2 seconds of data at receive_time = 100 seconds
    // time_ms: 2000 -> rounds to 2 seconds
    // sensor_start = 100 - 2 = 98 seconds
    let data1 = SensorData {
        total_pulses: 866, // 2 seconds worth
        time_ms: 2000,
    };
    let receive_time1: u64 = 100; // Unix timestamp in seconds
    let entries1 = process_sensor_data(data1, &mut state, 433, receive_time1);

    // Should have 2 entries: 98 and 99 seconds
    assert_eq!(
        entries1.len(),
        2,
        "Should have 2 entries for 2 seconds of data"
    );
    assert_eq!(entries1[0].timestamp, 98);
    assert_eq!(entries1[1].timestamp, 99);
    assert!(entries1[0].flow_rate_lpm > 0.0);
    assert!(entries1[1].flow_rate_lpm > 0.0);

    // Verify last_published_timestamp updated
    assert_eq!(state.last_published_timestamp, 99);

    // Second message: 3 seconds of data, received 5 seconds after last published
    // receive_time2 = 99 + 6 = 105 (need gap + 3s data, so at least 4s after)
    // time_ms: 3000 -> rounds to 3 seconds
    // sensor_start = 105 - 3 = 102
    // Gap: from 100 to 102 = 2 seconds (but last_published is 99, so gap is 100-101)
    let data2 = SensorData {
        total_pulses: 1299, // 3 seconds worth
        time_ms: 3000,
    };
    let receive_time2: u64 = 105; // 6 seconds after first message
    let entries2 = process_sensor_data(data2, &mut state, 433, receive_time2);

    // Should have: gap entries + 3 data entries
    // Gap: from 100 to 101 = 2 seconds of gap
    // Data: 102, 103, 104 = 3 seconds of data
    // Total: 5 entries
    assert_eq!(
        entries2.len(),
        5,
        "Should have 5 entries (2 gap + 3 data), got {:?}",
        entries2.iter().map(|e| e.timestamp).collect::<Vec<_>>()
    );

    // First 2 should be gap zeros
    assert_eq!(entries2[0].timestamp, 100);
    assert_eq!(entries2[0].flow_rate_lpm, 0.0);
    assert_eq!(entries2[1].timestamp, 101);
    assert_eq!(entries2[1].flow_rate_lpm, 0.0);

    // Last 3 should have actual flow
    assert_eq!(entries2[2].timestamp, 102);
    assert!(entries2[2].flow_rate_lpm > 0.0);
    assert_eq!(entries2[3].timestamp, 103);
    assert!(entries2[3].flow_rate_lpm > 0.0);
    assert_eq!(entries2[4].timestamp, 104);
    assert!(entries2[4].flow_rate_lpm > 0.0);

    // Verify last_published_timestamp updated to end of this data
    assert_eq!(state.last_published_timestamp, 104);
}

#[test]
fn test_variable_esp32_timing() {
    let mut state = ServiceState::new();

    // First message: ESP32 sends 1030ms (rounded to 1 second)
    // time_ms: 1030 -> (1030 + 500) / 1000 = 1 second
    // sensor_start = receive_time - seconds = 10 - 1 = 9
    let data1 = SensorData {
        total_pulses: 433,
        time_ms: 1030, // Variable timing from ESP32
    };
    let receive_time1: u64 = 10; // Unix timestamp in seconds
    let entries1 = process_sensor_data(data1, &mut state, 433, receive_time1);

    // Should have 1 entry at timestamp 9 (10 - 1)
    assert_eq!(
        entries1.len(),
        1,
        "Should have 1 entry for 1030ms rounded to 1 second"
    );
    assert_eq!(entries1[0].timestamp, 9);
    assert!(entries1[0].flow_rate_lpm > 0.0);
    assert_eq!(state.last_published_timestamp, 9);

    // Second message: ESP32 sends 1425ms (rounded to 1 second)
    // Received 1 second after first message
    // time_ms: 1425 -> (1425 + 500) / 1000 = 1 second (1925/1000 = 1)
    // sensor_start = 11 - 1 = 10
    // Gap: receive_time2(11) - last_published(9) - seconds(1) = 1 second gap
    // But gap is filled from 10 to 10 = 0 gap entries
    let data2 = SensorData {
        total_pulses: 433,
        time_ms: 1425, // Variable timing
    };
    let receive_time2: u64 = 11; // 1 second after first message
    let entries2 = process_sensor_data(data2, &mut state, 433, receive_time2);

    // Should have 1 entry at timestamp 10
    // No gap because: 11 - 9 - 1 = 1 second gap, but sensor_start = 10, which is last_published + 1
    assert_eq!(entries2.len(), 1, "Should handle variable 1425ms timing");
    assert_eq!(entries2[0].timestamp, 10);
    assert!(entries2[0].flow_rate_lpm > 0.0);
    assert_eq!(state.last_published_timestamp, 10);

    // Third message: ESP32 sends 15060ms (15 seconds rounded)
    // time_ms: 15060 -> (15060 + 500) / 1000 = 15 seconds
    // sensor_start = receive_time3 - 15 = 26 - 15 = 11
    // Gap: from 11 to 11 = no gap (continuous)
    let data3 = SensorData {
        total_pulses: 6495, // 15 seconds worth at 433 pulses/liter
        time_ms: 15060,     // Variable timing - 15 seconds
    };
    let receive_time3: u64 = 26; // 15 seconds after receive_time2
    let entries3 = process_sensor_data(data3, &mut state, 433, receive_time3);

    // Gap calculation: 26 - 10 - 15 = 1 second gap
    // sensor_start = 26 - 15 = 11
    // Gap from 11 to 11 = 0 seconds
    // So: 0 gap entries + 15 data entries = 15 total
    assert_eq!(
        entries3.len(),
        15,
        "Should handle 15-second burst, got {:?}",
        entries3.iter().map(|e| e.timestamp).collect::<Vec<_>>()
    );

    // First entry should be at 11 (data starts immediately after gap boundary)
    assert_eq!(entries3[0].timestamp, 11);
    assert!(entries3[0].flow_rate_lpm > 0.0);

    // Last entry should be at 25 (11 + 15 - 1 = 25)
    assert_eq!(entries3[14].timestamp, 25);
    assert!(entries3[14].flow_rate_lpm > 0.0);

    // Verify no duplicate timestamps (all should be unique and increasing)
    for i in 1..entries3.len() {
        assert!(
            entries3[i].timestamp > entries3[i - 1].timestamp,
            "Timestamps should always increase, no duplicates"
        );
    }

    // Verify last_published_timestamp updated correctly
    assert_eq!(state.last_published_timestamp, 25);
}
