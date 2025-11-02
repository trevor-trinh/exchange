/**
 * WebSocket message types for the exchange
 * Auto-generated from Rust types via ts-rs
 */

import type { components } from "./generated";

// Re-export REST API types
export type Trade = components["schemas"]["ApiTrade"];
export type Order = components["schemas"]["ApiOrder"];
export type Balance = components["schemas"]["ApiBalance"];

// Re-export WebSocket types from generated types
export type {
  ClientMessage,
  ServerMessage,
  SubscriptionChannel,
  TradeData,
  OrderbookData,
  PriceLevel as OrderbookLevel,
  Side,
  OrderType,
  OrderStatus,
} from "../../../shared/websocket";

// Message handler type
export type MessageHandler<T extends ServerMessage = ServerMessage> = (message: T) => void;
