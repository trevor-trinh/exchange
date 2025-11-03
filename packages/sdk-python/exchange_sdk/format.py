"""Internal formatting utilities for SDK.

These convert between atoms and display values.
"""

from decimal import Decimal


def to_display_value(atoms: str, decimals: int) -> float:
    """
    Convert atoms to display value.

    Args:
        atoms: Amount in atoms (smallest unit)
        decimals: Number of decimals for the token

    Returns:
        Human-readable decimal value
    """
    raw = Decimal(atoms)
    divisor = Decimal(10**decimals)
    return float(raw / divisor)


def format_number(value: float, max_decimals: int = 8) -> str:
    """
    Format a number with commas and appropriate decimals.

    Args:
        value: Number to format
        max_decimals: Maximum number of decimal places

    Returns:
        Formatted string
    """
    # Format with max decimals
    fixed = f"{value:.{max_decimals}f}"

    # Remove trailing zeros
    trimmed = fixed.rstrip("0").rstrip(".")

    # Split into integer and decimal parts
    parts = trimmed.split(".")
    integer = parts[0] if parts else "0"
    decimal = parts[1] if len(parts) > 1 else None

    # Add commas to integer part
    # Handle negative numbers
    is_negative = integer.startswith("-")
    if is_negative:
        integer = integer[1:]

    with_commas = f"{int(integer):,}"
    if is_negative:
        with_commas = f"-{with_commas}"

    # Rejoin with decimal if it exists
    return f"{with_commas}.{decimal}" if decimal else with_commas


def format_price(atoms: str, decimals: int) -> str:
    """
    Format a price value with smart precision.

    Args:
        atoms: Price in atoms
        decimals: Quote token decimals

    Returns:
        Formatted price string
    """
    value = to_display_value(atoms, decimals)

    # For high-value prices (>= 1000), always show 2 decimals
    if value >= 1000:
        return format_number(value, 2)

    # Otherwise use token decimals, capped at 8 for readability
    return format_number(value, min(decimals, 8))


def format_size(atoms: str, decimals: int) -> str:
    """
    Format a size value.

    Args:
        atoms: Size in atoms
        decimals: Base token decimals

    Returns:
        Formatted size string
    """
    value = to_display_value(atoms, decimals)
    return format_number(value, min(decimals, 8))


def to_atoms(value: str | float, decimals: int) -> str:
    """
    Convert display value to atoms.

    Args:
        value: Human-readable decimal value
        decimals: Number of decimals for the token

    Returns:
        Amount in atoms (string)
    """
    dec = Decimal(str(value))
    multiplier = Decimal(10**decimals)
    atoms = int(dec * multiplier)
    return str(atoms)
