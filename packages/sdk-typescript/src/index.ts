/**
 * Exchange SDK - TypeScript client for the exchange API
 *
 * @example
 * ```typescript
 * import { ExchangeClient } from '@exchange/sdk';
 *
 * const client = new ExchangeClient({
 *   restUrl: 'http://localhost:8888',
 *   wsUrl: 'ws://localhost:8888/ws',
 * });
 *
 * // REST API
 * const markets = await client.rest.getMarkets();
 *
 * // WebSocket
 * client.ws.connect();
 * client.ws.subscribe('Trades', { marketId: 'BTC/USDC' });
 * client.ws.on('trade', (message) => {
 *   console.log('New trade:', message.trade);
 * });
 * ```
 */

import { RestClient } from './rest';
import type { RestClientConfig, Trade, Order, Balance } from './rest';
import { WebSocketClient } from './websocket';
import type { WebSocketClientConfig } from './websocket';
import type { OrderbookLevel } from './types/websocket';

export { RestClient } from './rest';
export type { RestClientConfig } from './rest';

export { WebSocketClient } from './websocket';
export type { WebSocketClientConfig } from './websocket';

export {
  SdkError,
  ApiError,
  WebSocketError,
  ValidationError,
} from './errors';

// Export types
export type {
  Market,
  Token,
  Order,
  Trade,
  Balance,
  Side,
  OrderType,
  OrderStatus,
  Candle,
  CandlesResponse,
} from './rest';

export type {
  ClientMessage,
  ServerMessage,
  SubscriptionChannel,
  MessageHandler,
  OrderbookLevel,
} from './types/websocket';

// Re-export generated types for advanced usage
export type { components } from './types/generated';

/**
 * Main Exchange SDK client
 */
export interface ExchangeClientConfig {
  restUrl: string;
  wsUrl: string;
  restTimeout?: number;
  wsReconnectDelays?: number[];
  wsPingInterval?: number;
}

export class ExchangeClient {
  public readonly rest: RestClient;
  public readonly ws: WebSocketClient;

  constructor(config: string | ExchangeClientConfig) {
    // Support both simple URL string and full config object
    const cfg = typeof config === 'string'
      ? {
          restUrl: config,
          wsUrl: config.replace(/^http/, 'ws') + '/ws'
        }
      : config;

    this.rest = new RestClient({
      baseUrl: cfg.restUrl,
      timeout: cfg.restTimeout,
    });

    this.ws = new WebSocketClient({
      url: cfg.wsUrl,
      reconnectDelays: cfg.wsReconnectDelays,
      pingInterval: cfg.wsPingInterval,
    });
  }

  // ============================================================================
  // Convenience Methods - REST API
  // ============================================================================

  /**
   * Get all markets
   */
  getMarkets() {
    return this.rest.getMarkets();
  }

  /**
   * Get a specific market
   */
  getMarket(marketId: string) {
    return this.rest.getMarket(marketId);
  }

  /**
   * Get all tokens
   */
  getTokens() {
    return this.rest.getTokens();
  }

  /**
   * Get a specific token
   */
  getToken(ticker: string) {
    return this.rest.getToken(ticker);
  }

  /**
   * Get user balances
   */
  getBalances(userAddress: string) {
    return this.rest.getBalances(userAddress);
  }

  /**
   * Get user orders
   */
  getOrders(userAddress: string, marketId?: string) {
    return this.rest.getOrders({ userAddress, marketId });
  }

  /**
   * Get user trades
   */
  getTrades(userAddress: string, marketId?: string) {
    return this.rest.getTrades({ userAddress, marketId });
  }

  /**
   * Get candles (OHLCV data) for a market
   */
  getCandles(params: {
    marketId: string;
    interval: string;
    from: number;
    to: number;
  }) {
    return this.rest.getCandles(params);
  }

  /**
   * Place an order
   */
  placeOrder(params: {
    userAddress: string;
    marketId: string;
    side: 'buy' | 'sell';
    orderType: 'limit' | 'market';
    price: string;
    size: string;
    signature: string;
  }) {
    return this.rest.placeOrder(params);
  }

  /**
   * Cancel an order
   */
  cancelOrder(params: {
    userAddress: string;
    orderId: string;
    signature: string;
  }) {
    return this.rest.cancelOrder(params);
  }

  // ============================================================================
  // Type-Safe WebSocket Stream Helpers
  // ============================================================================

  /**
   * Stream trades for a market
   * @returns Unsubscribe function
   */
  onTrades(marketId: string, handler: (trade: Trade) => void): () => void {
    this.ws.connect();
    this.ws.subscribe('trades', { marketId });
    return this.ws.on('trade', (msg) => {
      if (msg.type === 'trade') {
        handler(msg.trade);
      }
    });
  }

  /**
   * Stream orderbook updates for a market
   * @returns Unsubscribe function
   */
  onOrderbook(marketId: string, handler: (update: { bids: OrderbookLevel[], asks: OrderbookLevel[] }) => void): () => void {
    this.ws.connect();
    this.ws.subscribe('orderbook', { marketId });
    return this.ws.on('orderbook_update', (msg) => {
      if (msg.type === 'orderbook_update') {
        handler({ bids: msg.bids, asks: msg.asks });
      }
    });
  }

  /**
   * Stream order updates for a user
   * @returns Unsubscribe function
   */
  onUserOrders(userAddress: string, handler: (order: Order) => void): () => void {
    this.ws.connect();
    this.ws.subscribe('user', { userAddress });
    return this.ws.on('order_update', (msg) => {
      if (msg.type === 'order_update') {
        handler(msg.order);
      }
    });
  }

  /**
   * Stream trade updates for a user
   * @returns Unsubscribe function
   */
  onUserTrades(userAddress: string, handler: (trade: Trade) => void): () => void {
    this.ws.connect();
    this.ws.subscribe('user', { userAddress });
    return this.ws.on('trade', (msg) => {
      if (msg.type === 'trade') {
        handler(msg.trade);
      }
    });
  }

  /**
   * Stream balance updates for a user
   * @returns Unsubscribe function
   */
  onUserBalances(userAddress: string, handler: (balance: Balance) => void): () => void {
    this.ws.connect();
    this.ws.subscribe('user', { userAddress });
    return this.ws.on('balance_update', (msg) => {
      if (msg.type === 'balance_update') {
        handler(msg.balance);
      }
    });
  }

  /**
   * Disconnect all connections
   */
  disconnect(): void {
    this.ws.disconnect();
  }
}
