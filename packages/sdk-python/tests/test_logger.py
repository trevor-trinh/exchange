"""Tests for logger."""

import pytest
from io import StringIO
import sys
from exchange_sdk import ConsoleLogger, NoopLogger, LogLevel


class TestConsoleLogger:
    """Test console logger functionality."""

    def test_log_levels(self, capsys):
        """Test different log levels."""
        logger = ConsoleLogger(level=LogLevel.INFO)

        # Debug should not show
        logger.debug("debug message")
        captured = capsys.readouterr()
        assert "debug message" not in captured.out

        # Info should show
        logger.info("info message")
        captured = capsys.readouterr()
        assert "info message" in captured.out

        # Warn should show
        logger.warn("warn message")
        captured = capsys.readouterr()
        assert "warn message" in captured.out

        # Error should show
        logger.error("error message")
        captured = capsys.readouterr()
        assert "error message" in captured.out

    def test_log_level_filtering(self, capsys):
        """Test that log level filtering works correctly."""
        logger = ConsoleLogger(level=LogLevel.ERROR)

        logger.debug("debug")
        logger.info("info")
        logger.warn("warn")
        logger.error("error")

        captured = capsys.readouterr()

        assert "debug" not in captured.out
        assert "info" not in captured.out
        assert "warn" not in captured.out
        assert "error" in captured.out

    def test_log_prefix(self, capsys):
        """Test custom log prefix."""
        logger = ConsoleLogger(level=LogLevel.INFO, prefix="[Custom]")

        logger.info("test message")
        captured = capsys.readouterr()

        assert "[Custom]" in captured.out
        assert "test message" in captured.out

    def test_set_level(self, capsys):
        """Test changing log level."""
        logger = ConsoleLogger(level=LogLevel.ERROR)

        logger.info("should not show")
        captured = capsys.readouterr()
        assert "should not show" not in captured.out

        # Change level
        logger.set_level(LogLevel.DEBUG)

        logger.info("should show")
        captured = capsys.readouterr()
        assert "should show" in captured.out

    def test_get_level(self):
        """Test getting log level."""
        logger = ConsoleLogger(level=LogLevel.WARN)
        assert logger.get_level() == LogLevel.WARN


class TestNoopLogger:
    """Test no-op logger."""

    def test_noop_logger(self, capsys):
        """Test that noop logger produces no output."""
        logger = NoopLogger()

        logger.debug("debug")
        logger.info("info")
        logger.warn("warn")
        logger.error("error")

        captured = capsys.readouterr()

        assert captured.out == ""
