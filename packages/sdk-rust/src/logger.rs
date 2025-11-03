//! Logging interface for the SDK
//!
//! Provides a trait-based logging system similar to TypeScript/Python SDKs

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// Logger trait that can be implemented for custom logging behavior
pub trait Logger: Send + Sync {
    fn debug(&self, message: &str);
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
    fn log(&self, level: LogLevel, message: &str);
}

/// Console logger that prints to stdout/stderr
#[derive(Debug, Clone)]
pub struct ConsoleLogger {
    level: LogLevel,
    prefix: String,
}

impl ConsoleLogger {
    pub fn new(level: LogLevel) -> Self {
        Self {
            level,
            prefix: "[Exchange SDK]".to_string(),
        }
    }

    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    fn should_log(&self, level: LogLevel) -> bool {
        level >= self.level
    }

    fn format_message(&self, level: LogLevel, message: &str) -> String {
        let level_str = match level {
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };
        format!("{} {}: {}", self.prefix, level_str, message)
    }
}

impl Logger for ConsoleLogger {
    fn debug(&self, message: &str) {
        if self.should_log(LogLevel::Debug) {
            println!("{}", self.format_message(LogLevel::Debug, message));
        }
    }

    fn info(&self, message: &str) {
        if self.should_log(LogLevel::Info) {
            println!("{}", self.format_message(LogLevel::Info, message));
        }
    }

    fn warn(&self, message: &str) {
        if self.should_log(LogLevel::Warn) {
            eprintln!("{}", self.format_message(LogLevel::Warn, message));
        }
    }

    fn error(&self, message: &str) {
        if self.should_log(LogLevel::Error) {
            eprintln!("{}", self.format_message(LogLevel::Error, message));
        }
    }

    fn log(&self, level: LogLevel, message: &str) {
        match level {
            LogLevel::Debug => self.debug(message),
            LogLevel::Info => self.info(message),
            LogLevel::Warn => self.warn(message),
            LogLevel::Error => self.error(message),
        }
    }
}

/// No-op logger that discards all log messages
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopLogger;

impl Logger for NoopLogger {
    fn debug(&self, _message: &str) {}
    fn info(&self, _message: &str) {}
    fn warn(&self, _message: &str) {}
    fn error(&self, _message: &str) {}
    fn log(&self, _level: LogLevel, _message: &str) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_ordering() {
        assert!(LogLevel::Debug < LogLevel::Info);
        assert!(LogLevel::Info < LogLevel::Warn);
        assert!(LogLevel::Warn < LogLevel::Error);
    }

    #[test]
    fn test_console_logger_filtering() {
        let logger = ConsoleLogger::new(LogLevel::Warn);
        assert!(!logger.should_log(LogLevel::Debug));
        assert!(!logger.should_log(LogLevel::Info));
        assert!(logger.should_log(LogLevel::Warn));
        assert!(logger.should_log(LogLevel::Error));
    }

    #[test]
    fn test_noop_logger() {
        let logger = NoopLogger;
        // Should not panic or do anything
        logger.debug("test");
        logger.info("test");
        logger.warn("test");
        logger.error("test");
    }
}
