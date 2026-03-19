//! FlowPulse MQTT Service
//!
//! A production-grade MQTT service that converts pulse sensor data into water flow rates
//! for integration with Home Assistant and other IoT platforms.
//!
//! # Architecture
//!
//! This service follows a clean architecture with clear separation of concerns:
//!
//! - **Service Layer** (`service`): Main business logic and MQTT event processing
//! - **Processing Layer** (`processor`): Flow rate calculations and gap filling
//! - **MQTT Layer** (`mqtt`): Abstracted MQTT client with testable traits
//! - **Time Layer** (`time`): Abstracted time for deterministic testing
//! - **State Management** (`state`): Thread-safe shared state for duplicate detection
//! - **Models** (`models`): Data structures for sensor input and timeseries output
//! - **Configuration** (`config`): Environment-based configuration
//!
//! # Key Features
//!
//! - **Gap Filling**: Automatically fills missing time periods with zero flow rates
//! - **Duplicate Prevention**: Tracks published timestamps to prevent duplicates
//! - **Auto-Reconnection**: Handles MQTT disconnections gracefully
//! - **Deterministic Testing**: Full test coverage with mocked dependencies
//! - **Static Binary**: No runtime dependencies for easy deployment
//!
//! # Testing Strategy
//!
//! The codebase supports multiple levels of testing:
//!
//! - **Unit Tests**: Individual function testing (in `#[cfg(test)]` modules)
//! - **Integration Tests**: Direct processor testing with various scenarios
//! - **Runtime Tests**: Full service testing with mocked MQTT and time
//!
//! # Example Usage
//!
//! ```rust,no_run
//! use custom_ha_service::config::Config;
//! use custom_ha_service::service::FlowPulseService;
//! use custom_ha_service::mqtt::RealMqttClient;
//! use custom_ha_service::time::SystemTimeProvider;
//!
//! async fn run_service() {
//!     let config = Config::from_env();
//!     let client = RealMqttClient::new(&config).await.unwrap();
//!     let service = FlowPulseService::new(config);
//!     let time_provider = SystemTimeProvider::new();
//!     
//!     service.run(client, time_provider).await.unwrap();
//! }
//! ```

// Core modules - production code
pub mod config;
pub mod models;
pub mod processor;
pub mod service;
pub mod state;

// Abstraction layers
pub mod mqtt;
pub mod time;

// Test utilities (public for integration tests)
pub mod test_utils;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Initialize logging with the specified level
/// 
/// This is a convenience function for applications using this library.
pub fn init_logging(level: log::Level) -> Result<(), log::SetLoggerError> {
    simple_logger::init_with_level(level)
}

/// Get current version string
pub fn version() -> &'static str {
    VERSION
}

/// Get service name
pub fn service_name() -> &'static str {
    NAME
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }

    #[test]
    fn test_service_name() {
        assert_eq!(service_name(), "custom-ha-service");
    }
}
