//! Testing utilities and mocks
//!
//! This module provides mock implementations and utilities for testing
//! the FlowPulse service without external dependencies.
//!
//! # Mock Implementations
//!
//! - **MockMqttClient**: Simulates MQTT broker with controllable behavior
//! - **LogCapture**: Captures log output for assertions
//! - **InMemoryMqttMock**: Legacy in-memory MQTT implementation

pub mod log_capture;
pub mod mock_mqtt;
pub mod mqtt_mock;

// Re-export for convenience
pub use log_capture::TestLogger;
pub use mock_mqtt::MockMqttClient;
pub use mqtt_mock::InMemoryMqttMock;
