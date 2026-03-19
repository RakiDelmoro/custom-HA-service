use log::{Level, Metadata, Record, SetLoggerError};
use std::sync::{Arc, Mutex};

/// A single log message entry
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct LogMessage {
    pub level: Level,
    pub message: String,
    pub target: String,
    pub timestamp: u64, // epoch milliseconds
}

#[allow(dead_code)]
impl LogMessage {
    pub fn contains(&self, pattern: &str) -> bool {
        self.message.contains(pattern)
    }

    pub fn matches_level(&self, level: Level) -> bool {
        self.level == level
    }
}

/// Logger implementation that captures logs for testing
#[allow(dead_code)]
pub struct TestLogger {
    messages: Arc<Mutex<Vec<LogMessage>>>,
    max_level: Level,
}

#[allow(dead_code)]
impl TestLogger {
    pub fn new(max_level: Level) -> Self {
        TestLogger {
            messages: Arc::new(Mutex::new(Vec::new())),
            max_level,
        }
    }

    /// Create a new logger with Info level
    pub fn new_with_info() -> Self {
        Self::new(Level::Info)
    }

    /// Create a new logger with Debug level
    pub fn new_with_debug() -> Self {
        Self::new(Level::Debug)
    }

    /// Get all captured log messages
    pub fn get_logs(&self) -> Vec<LogMessage> {
        self.messages.lock().unwrap().clone()
    }

    /// Get messages matching a specific pattern
    pub fn get_messages_containing(&self, pattern: &str) -> Vec<LogMessage> {
        self.messages
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.contains(pattern))
            .cloned()
            .collect()
    }

    /// Get messages at a specific level
    pub fn get_messages_at_level(&self, level: Level) -> Vec<LogMessage> {
        self.messages
            .lock()
            .unwrap()
            .iter()
            .filter(|m| m.matches_level(level))
            .cloned()
            .collect()
    }

    /// Check if any message contains the pattern
    pub fn has_message_containing(&self, pattern: &str) -> bool {
        self.messages
            .lock()
            .unwrap()
            .iter()
            .any(|m| m.contains(pattern))
    }

    /// Check if any message at specific level contains pattern
    pub fn has_message_at_level(&self, level: Level, pattern: &str) -> bool {
        self.messages
            .lock()
            .unwrap()
            .iter()
            .any(|m| m.matches_level(level) && m.contains(pattern))
    }

    /// Count total messages
    pub fn message_count(&self) -> usize {
        self.messages.lock().unwrap().len()
    }

    /// Clear all captured messages
    pub fn clear(&self) {
        self.messages.lock().unwrap().clear();
    }

    /// Get the last N messages
    pub fn get_last_n(&self, n: usize) -> Vec<LogMessage> {
        let messages = self.messages.lock().unwrap();
        let start = messages.len().saturating_sub(n);
        messages[start..].to_vec()
    }

    /// Print all captured messages (useful for debugging)
    pub fn print_all(&self) {
        for msg in self.get_logs() {
            println!("[{:?}] {}: {}", msg.level, msg.target, msg.message);
        }
    }
}

impl log::Log for TestLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = LogMessage {
                level: record.level(),
                message: record.args().to_string(),
                target: record.target().to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
            };
            self.messages.lock().unwrap().push(msg);
        }
    }

    fn flush(&self) {}
}

#[allow(dead_code)]
impl TestLogger {
    /// Initialize as global logger (can only be called once)
    #[allow(dead_code)]
    pub fn init(self) -> Result<(), SetLoggerError> {
        let max_level = self.max_level;
        log::set_boxed_logger(Box::new(self))?;
        log::set_max_level(max_level.to_level_filter());
        Ok(())
    }
}

/// Helper function to setup test logging
#[allow(dead_code)]
pub fn setup_test_logging() -> TestLogger {
    let logger = TestLogger::new_with_info();
    // Note: This will panic if logger already initialized
    // In tests, use lazy initialization or check first
    logger
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::Log;

    #[test]
    fn test_log_capture() {
        let logger = TestLogger::new_with_info();

        // Simulate logging
        let record = Record::builder()
            .args(format_args!("Test message"))
            .level(Level::Info)
            .target("test")
            .build();

        logger.log(&record);

        assert_eq!(logger.message_count(), 1);
        assert!(logger.has_message_containing("Test message"));
        assert!(logger.has_message_at_level(Level::Info, "Test message"));
    }

    #[test]
    fn test_level_filtering() {
        let logger = TestLogger::new(Level::Warn);

        // Info message should not be captured
        let info_record = Record::builder()
            .args(format_args!("Info message"))
            .level(Level::Info)
            .target("test")
            .build();

        logger.log(&info_record);
        assert_eq!(logger.message_count(), 0);

        // Warn message should be captured
        let warn_record = Record::builder()
            .args(format_args!("Warn message"))
            .level(Level::Warn)
            .target("test")
            .build();

        logger.log(&warn_record);
        assert_eq!(logger.message_count(), 1);
    }
}
