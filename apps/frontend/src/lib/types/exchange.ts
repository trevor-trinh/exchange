/**
 * Core exchange types
 */

export interface Market {
  id: string;
  base_ticker: string;
  quote_ticker: string;
  base_decimals: number;
  quote_decimals: number;
  tick_size: string;
  lot_size: string;
  min_size: string;
  maker_fee_bps: number;
  taker_fee_bps: number;
  created_at: string;
}

export interface Token {
  ticker: string;
  decimals: number;
  name: string;
}

export interface Balance {
  token_ticker: string;
  amount: number;
  locked: number;
  available: number;
}

export type Side = 'Buy' | 'Sell';
export type OrderType = 'Limit' | 'Market';
export type OrderStatus = 'Pending' | 'PartiallyFilled' | 'Filled' | 'Cancelled';

export interface Order {
  id: string;
  market_id: string;
  user_address: string;
  side: Side;
  order_type: OrderType;
  status: OrderStatus;
  price: string;
  size: string;
  filled_size: string;
  created_at: string;
  updated_at: string;
}

export interface Trade {
  id: string;
  market_id: string;
  buyer_address: string;
  seller_address: string;
  price: string;
  size: string;
  buyer_fee: string;
  seller_fee: string;
  timestamp: string;
}

// Orderbook types
export interface OrderbookLevel {
  price: string;
  size: string;
}

export interface Orderbook {
  market_id: string;
  bids: OrderbookLevel[];
  asks: OrderbookLevel[];
  timestamp?: number;
}

// For visualization
export interface PricePoint {
  timestamp: number;
  price: number;
}
