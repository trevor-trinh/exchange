//! Formatting utilities for converting between atoms and display values
//!
//! These utilities convert raw u128 values (atoms) to human-readable
//! display values and vice versa.

/// Convert atoms (u128) to display value (f64)
///
/// # Example
/// ```
/// use exchange_sdk::to_display_value;
/// // 1_000_000 atoms with 6 decimals = 1.0
/// assert_eq!(to_display_value(1_000_000, 6), 1.0);
/// // 123_456_789 atoms with 6 decimals = 123.456789
/// assert_eq!(to_display_value(123_456_789, 6), 123.456789);
/// ```
pub fn to_display_value(atoms: u128, decimals: u8) -> f64 {
    let divisor = 10u128.pow(decimals as u32);
    let whole_part = (atoms / divisor) as f64;
    let fractional_part = (atoms % divisor) as f64 / divisor as f64;
    whole_part + fractional_part
}

/// Convert display value (f64) to atoms (u128)
///
/// # Example
/// ```
/// use exchange_sdk::to_atoms;
/// // 1.0 with 6 decimals = 1_000_000 atoms
/// assert_eq!(to_atoms(1.0, 6), 1_000_000);
/// // 123.456789 with 6 decimals = 123_456_789 atoms
/// assert_eq!(to_atoms(123.456789, 6), 123_456_789);
/// ```
pub fn to_atoms(value: f64, decimals: u8) -> u128 {
    let multiplier = 10u128.pow(decimals as u32);
    (value * multiplier as f64).round() as u128
}

/// Format a number with commas and appropriate decimals
///
/// # Example
/// ```
/// use exchange_sdk::format_number;
/// assert_eq!(format_number(1234.5678, 2), "1,234.57");
/// assert_eq!(format_number(1000000.0, 2), "1,000,000");
/// ```
pub fn format_number(value: f64, max_decimals: u8) -> String {
    // Format with fixed decimals
    let formatted = format!("{:.prec$}", value, prec = max_decimals as usize);

    // Split into integer and decimal parts
    let parts: Vec<&str> = formatted.split('.').collect();
    let integer = parts[0];
    let decimal = parts.get(1).copied();

    // Add commas to integer part
    let with_commas = add_commas(integer);

    // Trim trailing zeros from decimal
    if let Some(dec) = decimal {
        let trimmed = dec.trim_end_matches('0');
        if trimmed.is_empty() {
            with_commas
        } else {
            format!("{}.{}", with_commas, trimmed)
        }
    } else {
        with_commas
    }
}

/// Add commas to an integer string
fn add_commas(s: &str) -> String {
    let bytes: Vec<char> = s.chars().collect();
    let mut result = String::new();
    let len = bytes.len();

    for (i, c) in bytes.iter().enumerate() {
        result.push(*c);
        let pos = len - i - 1;
        if pos > 0 && pos % 3 == 0 {
            result.push(',');
        }
    }

    result
}

/// Format a price value with smart precision
///
/// For high-value prices (>= 1000), always show 2 decimals (without trimming).
/// Otherwise use token decimals, capped at 8 for readability.
///
/// # Example
/// ```
/// use exchange_sdk::format_price;
/// assert_eq!(format_price(1_000_000_000, 6), "1,000.00");
/// assert_eq!(format_price(123_456_789, 6), "123.456789");
/// ```
pub fn format_price(atoms: u128, decimals: u8) -> String {
    let value = to_display_value(atoms, decimals);

    // For high-value prices (>= 1000), always show exactly 2 decimals
    if value >= 1000.0 {
        let formatted = format!("{:.2}", value);
        let parts: Vec<&str> = formatted.split('.').collect();
        let integer = parts[0];
        let decimal = parts.get(1).unwrap_or(&"00");
        format!("{}.{}", add_commas(integer), decimal)
    } else {
        // Otherwise use token decimals, capped at 8 for readability
        format_number(value, decimals.min(8))
    }
}

/// Format a size value
///
/// # Example
/// ```
/// use exchange_sdk::format_size;
/// assert_eq!(format_size(123_456_789, 6), "123.456789");
/// ```
pub fn format_size(atoms: u128, decimals: u8) -> String {
    let value = to_display_value(atoms, decimals);
    format_number(value, decimals.min(8))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_display_value() {
        assert_eq!(to_display_value(1_000_000, 6), 1.0);
        assert_eq!(to_display_value(123_456_789, 6), 123.456789);
        assert_eq!(to_display_value(0, 6), 0.0);
        assert_eq!(to_display_value(1, 6), 0.000001);
    }

    #[test]
    fn test_to_atoms() {
        assert_eq!(to_atoms(1.0, 6), 1_000_000);
        assert_eq!(to_atoms(123.456789, 6), 123_456_789);
        assert_eq!(to_atoms(0.0, 6), 0);
        assert_eq!(to_atoms(0.000001, 6), 1);
    }

    #[test]
    fn test_format_number() {
        assert_eq!(format_number(1234.5678, 2), "1,234.57");
        assert_eq!(format_number(1000000.0, 2), "1,000,000");
        assert_eq!(format_number(0.123, 3), "0.123");
        assert_eq!(format_number(1.0, 2), "1");
    }

    #[test]
    fn test_add_commas() {
        assert_eq!(add_commas("1000"), "1,000");
        assert_eq!(add_commas("1000000"), "1,000,000");
        assert_eq!(add_commas("123"), "123");
    }

    #[test]
    fn test_format_price() {
        assert_eq!(format_price(1_000_000_000, 6), "1,000.00");
        assert_eq!(format_price(123_456_789, 6), "123.456789");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(123_456_789, 6), "123.456789");
    }
}
