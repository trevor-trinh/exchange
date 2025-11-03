/**
 * WebSocket message types for the exchange
 * Auto-generated from Rust types via schemars
 */

import type { components } from "./generated";
import type { ServerMessage as WsServerMessage } from "./generated/websocket";

// Re-export REST API types
export type Trade = components["schemas"]["ApiTrade"];
export type Order = components["schemas"]["ApiOrder"];
export type Balance = components["schemas"]["ApiBalance"];

// Re-export WebSocket types from local generated types
export type {
  ClientMessage,
  ServerMessage,
  SubscriptionChannel,
  TradeData,
  OrderbookData,
  PriceLevel as OrderbookLevel,
  Side,
} from "./generated/websocket";

// Re-export REST API enum types (these come from OpenAPI schema)
export type OrderType = components["schemas"]["OrderType"];
export type OrderStatus = components["schemas"]["OrderStatus"];

// Message handler type
export type MessageHandler<T extends WsServerMessage = WsServerMessage> = (message: T) => void;
