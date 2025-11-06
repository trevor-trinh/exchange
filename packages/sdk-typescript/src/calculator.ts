/**
 * Order size calculator utilities
 */

import { toDisplayValue } from "./format";
import type { Market, Token } from "./cache";
import type { Side } from "./rest";

/**
 * Calculate maximum order size based on available balance
 */
export function calculateMaxSize(params: {
  side: Side;
  availableBase: number;
  availableQuote: number;
  currentPrice: number;
}): number {
  if (params.side === "buy") {
    // For buy orders: limited by quote balance / price
    return params.availableQuote / params.currentPrice;
  } else {
    // For sell orders: limited by base balance
    return params.availableBase;
  }
}

/**
 * Calculate size for a percentage of max balance
 */
export function calculatePercentageSize(params: {
  percentage: number;
  side: Side;
  availableBase: number;
  availableQuote: number;
  currentPrice: number;
  market: Market;
  baseToken: Token;
}): string {
  const maxSize = calculateMaxSize({
    side: params.side,
    availableBase: params.availableBase,
    availableQuote: params.availableQuote,
    currentPrice: params.currentPrice,
  });

  const targetSize = maxSize * (params.percentage / 100);
  const rounded = roundToLotSize(targetSize, params.market.lot_size, params.baseToken.decimals);

  // Get decimal places for display
  const decimals = getDecimalPlaces(params.market.lot_size, params.baseToken.decimals);

  return rounded.toFixed(decimals);
}

/**
 * Round a size to the nearest lot size
 */
export function roundToLotSize(size: number, lotSizeAtoms: string, baseDecimals: number): number {
  const lotValue = toDisplayValue(lotSizeAtoms, baseDecimals);
  if (lotValue === 0) return size;
  return Math.round(size / lotValue) * lotValue;
}

/**
 * Round a price to the nearest tick size
 */
export function roundToTickSize(price: number, tickSizeAtoms: string, quoteDecimals: number): number {
  const tickValue = toDisplayValue(tickSizeAtoms, quoteDecimals);
  if (tickValue === 0) return price;
  return Math.round(price / tickValue) * tickValue;
}

/**
 * Get number of decimal places needed to display a tick/lot size
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
