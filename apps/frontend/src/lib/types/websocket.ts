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
  | OrderbookSnapshotMessage
  | OrderbookUpdateMessage
  | OrderPlacedMessage
  | OrderCancelledMessage
  | OrderFilledMessage
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
  channel: "trades";
  data: {
    id: string;
    market_id: string;
    buyer_address: string;
    seller_address: string;
    price: string;
    size: string;
    timestamp: string;
  };
}

export interface OrderbookSnapshotMessage {
  type: "orderbook_snapshot";
  channel: "orderbook";
  data: {
    market_id: string;
    bids: Array<{ price: string; size: string }>;
    asks: Array<{ price: string; size: string }>;
  };
}

export interface OrderbookUpdateMessage {
  type: "orderbook_update";
  channel: "orderbook";
  data: {
    market_id: string;
    bids: Array<{ price: string; size: string }>;
    asks: Array<{ price: string; size: string }>;
  };
}

export interface OrderPlacedMessage {
  type: "order_placed";
  channel: "user";
  data: {
    user_address: string;
    market_id: string;
    order_id: string;
    side: "Buy" | "Sell";
    price: string;
    size: string;
  };
}

export interface OrderCancelledMessage {
  type: "order_cancelled";
  channel: "user";
  data: {
    user_address: string;
    order_id: string;
  };
}

export interface OrderFilledMessage {
  type: "order_filled";
  channel: "user";
  data: {
    user_address: string;
    order_id: string;
    filled_size: string;
  };
}

export interface ErrorMessage {
  type: "error";
  error: string;
  code?: string;
}
