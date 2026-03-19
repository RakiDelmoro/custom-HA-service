//! MQTT client abstraction layer
//!
//! This module provides a clean abstraction over MQTT operations,
//! allowing for both real MQTT connections and mock implementations for testing.
//!
//! # Architecture
//!
//! The module uses trait-based design for maximum testability:
//!
//! - **MqttClient trait**: Defines the interface for MQTT operations
//! - **MqttError**: Comprehensive error types for MQTT operations
//! - **MqttEvent**: Event types for MQTT polling
//!
//! # Implementations
//!
//! - **RealMqttClient**: Production implementation using rumqttc
//! - **MockMqttClient**: Test implementation with controllable behavior (in test_utils)

pub use client::{MqttClient, MqttError, MqttEvent};
pub use real::RealMqttClient;

mod client;
mod real;
