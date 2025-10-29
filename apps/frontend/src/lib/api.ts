/**
 * Exchange API client
 * Simplified to use @exchange/sdk directly
 */

import { ExchangeClient } from "@exchange/sdk";

// Singleton instance with auto-derived WebSocket URL
export const exchange = new ExchangeClient(
  process.env.NEXT_PUBLIC_API_URL || "http://localhost:8888"
);

// Re-export types and errors for convenience
export { ApiError } from "@exchange/sdk";
export type {
  Market,
  Token,
  Balance,
  Order,
  Trade,
  Side,
  OrderType,
  OrderStatus,
} from "@exchange/sdk";

// Export WebSocket types for real-time features
export type {
  SubscriptionChannel,
  OrderbookLevel,
} from "@exchange/sdk";
