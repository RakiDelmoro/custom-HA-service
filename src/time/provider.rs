//! Time provider trait and mock implementation

use std::sync::{Arc, Mutex};

/// Trait for time provider to allow mocking in tests
pub trait TimeProvider: Send + Sync {
    /// Get current time in seconds since epoch
    fn now_secs(&self) -> u64;

    /// Get current time in milliseconds since epoch
    #[allow(dead_code)]
    fn now_millis(&self) -> u64;
}

/// Mock time provider for tests
/// Allows setting and advancing time deterministically
#[derive(Clone)]
#[allow(dead_code)]
pub struct MockTimeProvider {
    current_time_ms: Arc<Mutex<u64>>,
}

#[allow(dead_code)]
impl MockTimeProvider {
    /// Creates a new MockTimeProvider with the specified initial time
    ///
    /// # Example
    ///
    /// ```rust
    /// use custom_ha_service::time::{MockTimeProvider, TimeProvider};
    ///
    /// let provider = MockTimeProvider::new(1000);
    /// assert_eq!(provider.now_secs(), 1);
    /// assert_eq!(provider.now_millis(), 1000);
    /// ```
    pub fn new(initial_time_ms: u64) -> Self {
        MockTimeProvider {
            current_time_ms: Arc::new(Mutex::new(initial_time_ms)),
        }
    }

    /// Set the current time to a specific value
    pub fn set_time(&self, time_ms: u64) {
        let mut time = self.current_time_ms.lock().unwrap();
        *time = time_ms;
    }

    /// Advance time by specified milliseconds
    pub fn advance(&self, delta_ms: u64) {
        let mut time = self.current_time_ms.lock().unwrap();
        *time += delta_ms;
    }

    /// Advance time by specified seconds (convenience method)
    pub fn advance_secs(&self, delta_secs: u64) {
        self.advance(delta_secs * 1000);
    }

    /// Get current time in milliseconds
    pub fn get_time_ms(&self) -> u64 {
        *self.current_time_ms.lock().unwrap()
    }
}

impl TimeProvider for MockTimeProvider {
    fn now_secs(&self) -> u64 {
        self.get_time_ms() / 1000
    }

    fn now_millis(&self) -> u64 {
        self.get_time_ms()
    }
}

impl Default for MockTimeProvider {
    fn default() -> Self {
        Self::new(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_time_provider_initial() {
        let provider = MockTimeProvider::new(1000);
        assert_eq!(provider.now_secs(), 1);
        assert_eq!(provider.now_millis(), 1000);
    }

    #[test]
    fn test_mock_time_advance() {
        let provider = MockTimeProvider::new(0);
        provider.advance(500);
        assert_eq!(provider.now_millis(), 500);
        assert_eq!(provider.now_secs(), 0);

        provider.advance(600);
        assert_eq!(provider.now_millis(), 1100);
        assert_eq!(provider.now_secs(), 1);
    }

    #[test]
    fn test_mock_time_advance_secs() {
        let provider = MockTimeProvider::new(1000);
        provider.advance_secs(5);
        assert_eq!(provider.now_millis(), 6000);
        assert_eq!(provider.now_secs(), 6);
    }

    #[test]
    fn test_mock_time_set() {
        let provider = MockTimeProvider::new(0);
        provider.set_time(50000);
        assert_eq!(provider.now_millis(), 50000);
        assert_eq!(provider.now_secs(), 50);
    }
}
