export type ClientMessage =
  | {
      channel: SubscriptionChannel;
      market_id?: string | null;
      type: "subscribe";
      user_address?: string | null;
    }
  | {
      channel: SubscriptionChannel;
      market_id?: string | null;
      type: "unsubscribe";
      user_address?: string | null;
    }
  | {
      type: "ping";
    };
/**
 * Channel types for WebSocket subscriptions
 */

export type SubscriptionChannel = "trades" | "orderbook" | "user_fills" | "user_orders" | "user_balances";

export interface OrderbookData {
  asks: PriceLevel[];
  bids: PriceLevel[];
  market_id: string;
}

export interface PriceLevel {
  price: string;
  size: string;
}

export type ServerMessage =
  | {
      channel: SubscriptionChannel;
      market_id?: string | null;
      type: "subscribed";
      user_address?: string | null;
    }
  | {
      channel: SubscriptionChannel;
      market_id?: string | null;
      type: "unsubscribed";
      user_address?: string | null;
    }
  | {
      trade: TradeData;
      type: "trade";
    }
  | {
      orderbook: OrderbookData;
      type: "orderbook";
    }
  | {
      close: string;
      high: string;
      low: string;
      market_id: string;
      open: string;
      timestamp: number;
      type: "candle";
      volume: string;
    }
  | {
      trade: TradeData;
      type: "user_fill";
    }
  | {
      filled_size: string;
      order_id: string;
      status: string;
      type: "user_order";
    }
  | {
      available: string;
      locked: string;
      token_ticker: string;
      type: "user_balance";
      updated_at: number;
      user_address: string;
    }
  | {
      message: string;
      type: "error";
    }
  | {
      type: "pong";
    };
/**
 * Channel types for WebSocket subscriptions
 */

export type Side = "buy" | "sell";

/**
 * Trade data for WebSocket messages (API layer with String fields)
 */

export interface TradeData {
  buyer_address: string;
  buyer_order_id: string;
  id: string;
  market_id: string;
  price: string;
  seller_address: string;
  seller_order_id: string;
  side: Side;
  size: string;
  timestamp: number;
}
