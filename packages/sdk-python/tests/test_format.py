"""Tests for format utilities."""

import pytest
from exchange_sdk import (
    to_display_value,
    to_atoms,
    format_price,
    format_size,
    format_number,
)


class TestFormatUtilities:
    """Test format utilities for atom conversion."""

    def test_to_display_value(self):
        """Test converting atoms to display value."""
        # 6 decimals (USDC)
        assert to_display_value("1000000", 6) == 1.0
        assert to_display_value("110000500000", 6) == 110000.50
        assert to_display_value("500000", 6) == 0.5

        # 8 decimals (BTC)
        assert to_display_value("100000000", 8) == 1.0
        assert to_display_value("50000000", 8) == 0.5
        assert to_display_value("1", 8) == 0.00000001

    def test_to_atoms(self):
        """Test converting decimal to atoms."""
        # 6 decimals (USDC)
        assert to_atoms("1.0", 6) == "1000000"
        assert to_atoms("110000.50", 6) == "110000500000"
        assert to_atoms("0.5", 6) == "500000"

        # 8 decimals (BTC)
        assert to_atoms("1.0", 8) == "100000000"
        assert to_atoms("0.5", 8) == "50000000"
        assert to_atoms("0.00000001", 8) == "1"

    def test_format_number(self):
        """Test number formatting with commas."""
        assert format_number(1000.0) == "1,000"
        assert format_number(110000.50) == "110,000.5"
        assert format_number(0.5) == "0.5"
        assert format_number(0.00000001, 8) == "0.00000001"

        # Remove trailing zeros
        assert format_number(1.00000000, 8) == "1"
        assert format_number(0.50000000, 8) == "0.5"

    def test_format_price(self):
        """Test price formatting."""
        # High prices (>= 1000) always show 2 decimals
        assert format_price("110000500000", 6) == "110,000.5"
        assert format_price("1000000000", 6) == "1,000"

        # Low prices use token decimals (capped at 8)
        assert format_price("500000", 6) == "0.5"
        assert format_price("1", 8) == "0.00000001"

    def test_format_size(self):
        """Test size formatting."""
        assert format_size("100000000", 8) == "1"
        assert format_size("50000000", 8) == "0.5"
        assert format_size("1", 8) == "0.00000001"

    def test_roundtrip_conversion(self):
        """Test converting decimal -> atoms -> decimal."""
        original = "110000.50"
        atoms = to_atoms(original, 6)
        back_to_decimal = to_display_value(atoms, 6)
        assert back_to_decimal == 110000.50

        # BTC
        original = "0.5"
        atoms = to_atoms(original, 8)
        back_to_decimal = to_display_value(atoms, 8)
        assert back_to_decimal == 0.5
