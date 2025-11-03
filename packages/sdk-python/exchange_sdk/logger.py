"""Logging interface and implementations."""

from abc import ABC, abstractmethod
from enum import Enum
from typing import Any


class LogLevel(str, Enum):
    """Log level enumeration."""

    DEBUG = "debug"
    INFO = "info"
    WARN = "warn"
    ERROR = "error"
    NONE = "none"


class Logger(ABC):
    """Abstract logger interface."""

    @abstractmethod
    def debug(self, message: str, *args: Any) -> None:
        """Log debug message."""
        pass

    @abstractmethod
    def info(self, message: str, *args: Any) -> None:
        """Log info message."""
        pass

    @abstractmethod
    def warn(self, message: str, *args: Any) -> None:
        """Log warning message."""
        pass

    @abstractmethod
    def error(self, message: str, *args: Any) -> None:
        """Log error message."""
        pass


class ConsoleLogger(Logger):
    """Console logger implementation with configurable log levels."""

    _LEVELS = {
        LogLevel.DEBUG: 0,
        LogLevel.INFO: 1,
        LogLevel.WARN: 2,
        LogLevel.ERROR: 3,
        LogLevel.NONE: 4,
    }

    def __init__(self, level: LogLevel = LogLevel.INFO, prefix: str = "[Exchange SDK]"):
        """
        Initialize console logger.

        Args:
            level: Minimum log level to display
            prefix: Prefix for log messages
        """
        self.level = level
        self.prefix = prefix

    def _should_log(self, level: LogLevel) -> bool:
        """Check if message should be logged based on level."""
        return self._LEVELS[level] >= self._LEVELS[self.level]

    def _format_message(self, level: str, message: str) -> str:
        """Format log message with prefix and level."""
        return f"{self.prefix} {level.upper()}: {message}"

    def debug(self, message: str, *args: Any) -> None:
        """Log debug message."""
        if self._should_log(LogLevel.DEBUG):
            print(self._format_message("debug", message), *args)

    def info(self, message: str, *args: Any) -> None:
        """Log info message."""
        if self._should_log(LogLevel.INFO):
            print(self._format_message("info", message), *args)

    def warn(self, message: str, *args: Any) -> None:
        """Log warning message."""
        if self._should_log(LogLevel.WARN):
            print(self._format_message("warn", message), *args)

    def error(self, message: str, *args: Any) -> None:
        """Log error message."""
        if self._should_log(LogLevel.ERROR):
            print(self._format_message("error", message), *args)

    def set_level(self, level: LogLevel) -> None:
        """Set log level."""
        self.level = level

    def get_level(self) -> LogLevel:
        """Get current log level."""
        return self.level


class NoopLogger(Logger):
    """No-op logger that discards all log messages."""

    def debug(self, message: str, *args: Any) -> None:
        """Do nothing."""
        pass

    def info(self, message: str, *args: Any) -> None:
        """Do nothing."""
        pass

    def warn(self, message: str, *args: Any) -> None:
        """Do nothing."""
        pass

    def error(self, message: str, *args: Any) -> None:
        """Do nothing."""
        pass
