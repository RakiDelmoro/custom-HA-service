//! System time provider implementation

use super::provider::TimeProvider;
use std::time::{SystemTime, UNIX_EPOCH};

/// Real time provider using SystemTime
pub struct SystemTimeProvider;

impl SystemTimeProvider {
    /// Creates a new SystemTimeProvider
    ///
    /// # Example
    ///
    /// ```rust
    /// use custom_ha_service::time::{SystemTimeProvider, TimeProvider};
    ///
    /// let provider = SystemTimeProvider::new();
    /// let secs = provider.now_secs();
    /// assert!(secs > 0); // Should be positive
    /// ```
    pub fn new() -> Self {
        SystemTimeProvider
    }
}

impl Default for SystemTimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeProvider for SystemTimeProvider {
    fn now_secs(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    fn now_millis(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_time_provider() {
        let provider = SystemTimeProvider::new();
        let secs = provider.now_secs();
        let millis = provider.now_millis();

        // Basic sanity checks
        assert!(secs > 0);
        assert!(millis > 0);
        assert!(millis >= secs * 1000);
        assert!(millis < (secs + 1) * 1000);
    }
}
