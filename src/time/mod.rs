//! Time provider abstraction layer
//!
//! This module provides an abstraction over time operations,
//! allowing for deterministic testing by mocking time.
//!
//! # Architecture
//!
//! The module uses trait-based design for maximum testability:
//!
//! - **TimeProvider trait**: Defines the interface for time operations
//! - **SystemTimeProvider**: Production implementation using SystemTime
//! - **MockTimeProvider**: Test implementation with controllable time

pub use provider::{MockTimeProvider, TimeProvider};
pub use system::SystemTimeProvider;

mod provider;
mod system;
