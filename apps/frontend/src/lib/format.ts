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
 * Format a price for display with appropriate precision
 * @param rawPrice Raw price string from backend
 * @param quoteDecimals Number of decimals for the quote token
 * @returns Formatted price string
 */
export function formatPrice(rawPrice: string, quoteDecimals: number): string {
  const price = parsePrice(rawPrice, quoteDecimals);
  // Use quote_decimals for display precision, but cap at 8 for readability
  const decimals = Math.min(quoteDecimals, 8);
  return price.toFixed(decimals);
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
  return size.toFixed(decimals);
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
