/**
 * Internal formatting utilities for SDK
 * These convert between atoms and display values
 */

/**
 * Convert atoms to display value
 */
export function toDisplayValue(atoms: string, decimals: number): number {
  const raw = BigInt(atoms);
  const divisor = BigInt(10 ** decimals);
  const wholePart = Number(raw / divisor);
  const fractionalPart = Number(raw % divisor) / Number(divisor);
  return wholePart + fractionalPart;
}

/**
 * Convert display value to atoms (raw value)
 */
export function toRawValue(displayValue: number | string, decimals: number): string {
  const value = typeof displayValue === "string" ? parseFloat(displayValue) : displayValue;
  if (isNaN(value)) return "0";

  // Use BigInt for precision
  const multiplier = BigInt(10 ** decimals);
  const wholePart = Math.floor(value);
  const fractionalPart = value - wholePart;

  const wholeRaw = BigInt(wholePart) * multiplier;
  const fractionalRaw = BigInt(Math.round(fractionalPart * Number(multiplier)));

  return (wholeRaw + fractionalRaw).toString();
}

/**
 * Format a number with commas and appropriate decimals
 */
export function formatNumber(value: number, maxDecimals: number = 8, keepTrailingZeros: boolean = false): string {
  // Format with max decimals
  const fixed = value.toFixed(maxDecimals);

  // Remove trailing zeros (unless keepTrailingZeros is true)
  const trimmed = keepTrailingZeros ? fixed : fixed.replace(/\.?0+$/, "");

  // Split into integer and decimal parts
  const parts = trimmed.split(".");
  const integer = parts[0] || "0";
  const decimal = parts[1];

  // Add commas to integer part
  const withCommas = integer.replace(/\B(?=(\d{3})+(?!\d))/g, ",");

  // Rejoin with decimal if it exists
  return decimal !== undefined ? `${withCommas}.${decimal}` : withCommas;
}

/**
 * Format a price value with smart precision
 */
export function formatPrice(atoms: string, decimals: number): string {
  const value = toDisplayValue(atoms, decimals);

  // For high-value prices (>= 1000), always show 2 decimals
  if (value >= 1000) {
    return formatNumber(value, 2);
  }

  // Otherwise use token decimals, capped at 8 for readability
  return formatNumber(value, Math.min(decimals, 8));
}

/**
 * Format a size value
 */
export function formatSize(atoms: string, decimals: number): string {
  const value = toDisplayValue(atoms, decimals);
  return formatNumber(value, Math.min(decimals, 8));
}
