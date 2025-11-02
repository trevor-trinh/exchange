/**
 * Formatting utilities for market data with proper decimal handling
 */

import type { Market, Token } from "./types/exchange";

/**
 * Convert a raw price value to actual value using quote token decimals
 * @param rawPrice Raw price string from backend (fixed-point integer)
 * @param quoteDecimals Number of decimals for the quote token
 * @returns Actual price as a number
 */
export function parsePrice(rawPrice: string, quoteDecimals: number): number {
  const raw = parseFloat(rawPrice);
  return raw / Math.pow(10, quoteDecimals);
}

/**
 * Convert a raw size value to actual value using base token decimals
 * @param rawSize Raw size string from backend (fixed-point integer)
 * @param baseDecimals Number of decimals for the base token
 * @returns Actual size as a number
 */
export function parseSize(rawSize: string, baseDecimals: number): number {
  const raw = parseFloat(rawSize);
  return raw / Math.pow(10, baseDecimals);
}

/**
 * Format a number with commas and remove trailing zeros
 * @param value Number to format
 * @param maxDecimals Maximum number of decimal places
 * @param keepTrailingZeros If true, keeps trailing zeros (e.g., for prices)
 * @returns Formatted string with commas
 */
export function formatNumberWithCommas(
  value: number,
  maxDecimals: number = 8,
  keepTrailingZeros: boolean = false,
): string {
  // Format with max decimals
  const fixed = value.toFixed(maxDecimals);

  // If we want to keep trailing zeros, don't trim them
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
 * Format a price for display with appropriate precision
 * @param rawPrice Raw price string from backend
 * @param quoteDecimals Number of decimals for the quote token
 * @returns Formatted price string
 */
export function formatPrice(rawPrice: string, quoteDecimals: number): string {
  const price = parsePrice(rawPrice, quoteDecimals);

  // For high-value prices (>= 1000, like BTC), always show 2 decimals
  if (price >= 1000) {
    return formatNumberWithCommas(price, 2, true);
  }

  // Use quote_decimals for display precision, but cap at 8 for readability
  const decimals = Math.min(quoteDecimals, 8);
  return formatNumberWithCommas(price, decimals);
}

/**
 * Format a size for display with appropriate precision
 * @param rawSize Raw size string from backend
 * @param baseDecimals Number of decimals for the base token
 * @returns Formatted size string
 */
export function formatSize(rawSize: string, baseDecimals: number): string {
  const size = parseSize(rawSize, baseDecimals);
  // Use base_decimals for display precision, but cap at 8 for readability
  const decimals = Math.min(baseDecimals, 8);
  return formatNumberWithCommas(size, decimals);
}

/**
 * Format a number with compact notation for large numbers
 * @param value Number to format
 * @param decimals Number of decimal places
 * @returns Formatted string
 */
export function formatCompact(value: number, decimals: number = 2): string {
  if (value >= 1_000_000) {
    return `${(value / 1_000_000).toFixed(decimals)}M`;
  } else if (value >= 1_000) {
    return `${(value / 1_000).toFixed(decimals)}K`;
  }
  return value.toFixed(decimals);
}

/**
 * Get a display-friendly price with the right precision based on price magnitude
 * @param price Price value
 * @param maxDecimals Maximum decimals to show
 * @returns Formatted price string
 */
export function formatDisplayPrice(price: number, maxDecimals: number = 8): string {
  if (price >= 1000) {
    return price.toFixed(2); // Large prices: 2 decimals
  } else if (price >= 1) {
    return price.toFixed(Math.min(4, maxDecimals)); // Medium prices: 4 decimals
  } else {
    return price.toFixed(Math.min(6, maxDecimals)); // Small prices: 6 decimals
  }
}

/**
 * Convert display value to raw value (multiply by 10^decimals)
 * @param displayValue User-friendly number
 * @param decimals Token decimals
 * @returns Raw value as string
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
 * Convert raw value to display value (divide by 10^decimals)
 * @param rawValue Raw value string
 * @param decimals Token decimals
 * @returns Display value as number
 */
export function toDisplayValue(rawValue: string, decimals: number): number {
  const raw = BigInt(rawValue);
  const divisor = BigInt(10 ** decimals);
  const wholePart = Number(raw / divisor);
  const fractionalPart = Number(raw % divisor) / Number(divisor);
  return wholePart + fractionalPart;
}

/**
 * Round a price to the nearest tick size
 * @param price Display price
 * @param tickSize Raw tick size from market
 * @param quoteDecimals Quote token decimals
 * @returns Rounded display price
 */
export function roundToTickSize(price: number, tickSize: string, quoteDecimals: number): number {
  const tickValue = toDisplayValue(tickSize, quoteDecimals);
  if (tickValue === 0) return price;
  return Math.round(price / tickValue) * tickValue;
}

/**
 * Round a size to the nearest lot size
 * @param size Display size
 * @param lotSize Raw lot size from market
 * @param baseDecimals Base token decimals
 * @returns Rounded display size
 */
export function roundToLotSize(size: number, lotSize: string, baseDecimals: number): number {
  const lotValue = toDisplayValue(lotSize, baseDecimals);
  if (lotValue === 0) return size;
  return Math.round(size / lotValue) * lotValue;
}

/**
 * Get the number of decimal places needed to display a tick/lot size
 * @param tickOrLotSize Raw tick/lot size
 * @param decimals Token decimals
 * @returns Number of decimal places
 */
export function getDecimalPlaces(tickOrLotSize: string, decimals: number): number {
  const displayValue = toDisplayValue(tickOrLotSize, decimals);
  if (displayValue === 0) return decimals;

  const str = displayValue.toFixed(decimals);
  const trimmed = str.replace(/\.?0+$/, "");
  const decimalIndex = trimmed.indexOf(".");

  if (decimalIndex === -1) return 0;
  return trimmed.length - decimalIndex - 1;
}

/**
 * Format a timestamp to localized time string
 * @param timestamp Date object, ISO string, or Unix timestamp (ms)
 * @returns Formatted time string (e.g., "2:30:45 PM")
 */
export function formatTime(timestamp: Date | string | number): string {
  const date = typeof timestamp === "number" || typeof timestamp === "string" ? new Date(timestamp) : timestamp;
  return date.toLocaleTimeString();
}

/**
 * Format a timestamp to localized date string
 * @param timestamp Date object, ISO string, or Unix timestamp (ms)
 * @returns Formatted date string (e.g., "12/31/2024")
 */
export function formatDate(timestamp: Date | string | number): string {
  const date = typeof timestamp === "number" || typeof timestamp === "string" ? new Date(timestamp) : timestamp;
  return date.toLocaleDateString();
}

/**
 * Format a timestamp to localized date and time string
 * @param timestamp Date object, ISO string, or Unix timestamp (ms)
 * @returns Formatted date and time string (e.g., "12/31/2024, 2:30:45 PM")
 */
export function formatDateTime(timestamp: Date | string | number): string {
  const date = typeof timestamp === "number" || typeof timestamp === "string" ? new Date(timestamp) : timestamp;
  return date.toLocaleString();
}

/**
 * Format a number with a maximum number of decimals, removing trailing zeros
 * @param value Number to format
 * @param maxDecimals Maximum number of decimal places
 * @returns Formatted string without trailing zeros
 */
export function formatWithoutTrailingZeros(value: number, maxDecimals: number): string {
  const fixed = value.toFixed(maxDecimals);
  return parseFloat(fixed).toString();
}
