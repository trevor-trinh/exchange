/**
 * WebSocket message types for the exchange
 * These are hand-written since WebSocket isn't part of OpenAPI
 */

import type { components } from "./generated";

// Re-export domain types from generated
export type Trade = components["schemas"]["ApiTrade"];
export type Order = components["schemas"]["ApiOrder"];
export type Balance = components["schemas"]["ApiBalance"];

// Subscription channels (must match backend snake_case serialization)
export type SubscriptionChannel = "trades" | "orderbook" | "user";

// Client -> Server messages
export type ClientMessage =
  | {
      type: "subscribe";
      channel: SubscriptionChannel;
      market_id?: string;
      user_address?: string;
    }
  | {
      type: "unsubscribe";
      channel: SubscriptionChannel;
      market_id?: string;
      user_address?: string;
    }
  | {
      type: "ping";
    };

// Server -> Client messages
export type ServerMessage =
  | {
      type: "subscribed";
      channel: SubscriptionChannel;
      market_id?: string;
      user_address?: string;
    }
  | {
      type: "unsubscribed";
      channel: SubscriptionChannel;
      market_id?: string;
      user_address?: string;
    }
  | {
      type: "trade";
      trade: TradeData;
    }
  | {
      type: "orderbook";
      orderbook: OrderbookData;
    }
  | {
      type: "order";
      order_id: string;
      status: string;
      filled_size: string;
    }
  | {
      type: "balance";
      token_ticker: string;
      available: string;
      locked: string;
    }
  | {
      type: "candle";
      market_id: string;
      timestamp: number;
      open: string;
      high: string;
      low: string;
      close: string;
      volume: string;
    }
  | {
      type: "pong";
    }
  | {
      type: "error";
      message: string;
    };

// Trade data structure
export interface TradeData {
  id: string;
  market_id: string;
  buyer_address: string;
  seller_address: string;
  buyer_order_id: string;
  seller_order_id: string;
  price: string;
  size: string;
  side: "buy" | "sell"; // Taker's side (determines if trade is "buy" or "sell" on tape)
  timestamp: number;
}

// Orderbook data structure
export interface OrderbookData {
  market_id: string;
  bids: OrderbookLevel[];
  asks: OrderbookLevel[];
}

export interface OrderbookLevel {
  price: string;
  size: string;
}

// Message handler type
export type MessageHandler<T extends ServerMessage = ServerMessage> = (message: T) => void;
