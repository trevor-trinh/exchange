/**
 * WebSocket message types for the exchange
 */

// ============================================================================
// Client → Server Messages
// ============================================================================

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

export type SubscriptionChannel = "trades" | "orderbook" | "user";

// ============================================================================
// Server → Client Messages
// ============================================================================

export type ServerMessage =
  | SubscribedMessage
  | UnsubscribedMessage
  | PongMessage
  | TradeMessage
  | OrderbookMessage
  | OrderMessage
  | BalanceMessage
  | CandleMessage
  | ErrorMessage;

export interface SubscribedMessage {
  type: "subscribed";
  channel: string;
  market_id?: string;
  user_address?: string;
}

export interface UnsubscribedMessage {
  type: "unsubscribed";
  channel: string;
  market_id?: string;
  user_address?: string;
}

export interface PongMessage {
  type: "pong";
  timestamp: number;
}

export interface TradeMessage {
  type: "trade";
  trade: {
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
  };
}

export interface OrderbookMessage {
  type: "orderbook";
  orderbook: {
    market_id: string;
    bids: Array<{ price: string; size: string }>;
    asks: Array<{ price: string; size: string }>;
  };
}

export interface OrderMessage {
  type: "order";
  order_id: string;
  status: string;
  filled_size: string;
}

export interface BalanceMessage {
  type: "balance";
  token_ticker: string;
  available: string;
  locked: string;
}

export interface CandleMessage {
  type: "candle";
  market_id: string;
  timestamp: number;
  open: string;
  high: string;
  low: string;
  close: string;
  volume: string;
}

export interface ErrorMessage {
  type: "error";
  error: string;
  code?: string;
}
