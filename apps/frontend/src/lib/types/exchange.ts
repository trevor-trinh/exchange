/**
 * Core exchange types - now imported from SDK
 */

// Re-export SDK types
export type {
  Market,
  Token,
  Side,
  OrderType,
  OrderStatus,
  // Use enhanced types from SDK
  EnhancedTrade as Trade,
  EnhancedOrder as Order,
  EnhancedBalance as Balance,
  EnhancedOrderbookLevel as OrderbookLevel,
} from "@exchange/sdk";

// Orderbook composite type - uses enhanced data from SDK
export interface Orderbook {
  market_id: string;
  bids: import("@exchange/sdk").EnhancedOrderbookLevel[];
  asks: import("@exchange/sdk").EnhancedOrderbookLevel[];
  timestamp?: number;
}

// For visualization
export interface PricePoint {
  timestamp: number;
  price: number;
}
