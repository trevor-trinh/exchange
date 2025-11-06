/**
 * Formatting utilities - mostly re-exports from SDK
 * Only keep functions here if they're not in the SDK
 */

// Re-export SDK utilities
export { toDisplayValue, formatNumber } from "@exchange/sdk";

/**
 * Format a number with a maximum number of decimals, removing trailing zeros
 * Simple wrapper around parseFloat
 */
export function formatWithoutTrailingZeros(value: number, maxDecimals: number): string {
  const fixed = value.toFixed(maxDecimals);
  return parseFloat(fixed).toString();
}
