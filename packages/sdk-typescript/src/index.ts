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
import type { RestClientConfig, Trade, Order, Balance, EnhancedTrade, EnhancedOrder, EnhancedBalance, EnhancedOrderbookLevel } from './rest';
import { WebSocketClient } from './websocket';
import type { WebSocketClientConfig } from './websocket';
import type { OrderbookLevel, TradeData, ServerMessage } from './types/websocket';

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
  EnhancedTrade,
  EnhancedOrder,
  EnhancedBalance,
  EnhancedOrderbookLevel,
} from './rest';

export type {
  ClientMessage,
  ServerMessage,
  SubscriptionChannel,
  MessageHandler,
  OrderbookLevel,
  TradeData,
  OrderbookData,
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

  private cacheInitialized = false;
  private cacheInitPromise: Promise<void> | null = null;

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

    // Initialize cache and connect WebSocket immediately
    this.initialize();
  }

  /**
   * Initialize the SDK (called automatically in constructor)
   * @private
   */
  private initialize(): void {
    // Start cache initialization
    this.initializeCache().catch(error => {
      console.error('[SDK] Failed to initialize cache:', error);
    });

    // Connect WebSocket
    this.ws.connect();
  }

  /**
   * Initialize the SDK cache with markets and tokens
   * Called automatically in constructor
   * @public - Can also be called manually to force refresh
   */
  async initializeCache(): Promise<void> {
    // If already initialized, return immediately
    if (this.cacheInitialized) {
      return;
    }

    // If initialization is in progress, wait for it
    if (this.cacheInitPromise) {
      return this.cacheInitPromise;
    }

    console.log('[SDK] Starting cache initialization...');

    // Start initialization
    this.cacheInitPromise = (async () => {
      try {
        const [markets, tokens] = await Promise.all([
          this.rest.getMarkets(),
          this.rest.getTokens(),
        ]);
        this.cacheInitialized = true;
        console.log(`[SDK] Cache initialized: ${markets.length} markets, ${tokens.length} tokens`);
      } catch (error) {
        console.error('[SDK] Failed to initialize cache:', error);
        this.cacheInitPromise = null; // Allow retry
        throw error;
      }
    })();

    return this.cacheInitPromise;
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

  /**
   * Cancel all orders for a user, optionally filtered by market
   */
  cancelAllOrders(params: {
    userAddress: string;
    marketId?: string;
    signature: string;
  }) {
    return this.rest.cancelAllOrders(params);
  }

  // ============================================================================
  // Type-Safe WebSocket Stream Helpers
  // ============================================================================

  /**
   * Stream trades for a market (enhanced with display values)
   * @returns Unsubscribe function
   */
  onTrades(marketId: string, handler: (trade: EnhancedTrade) => void): () => void {
    console.log(`[SDK] onTrades called for ${marketId}`);

    // Register WebSocket message handler
    const removeHandler = this.ws.on('trade', (msg) => {
      if (msg.type !== 'trade' || msg.trade.market_id !== marketId) return;

      // Wait for cache if not ready yet
      if (!this.cacheInitialized) {
        console.warn('[SDK] Trade received before cache initialized, skipping');
        return;
      }

      // Convert WebSocket TradeData to REST Trade format
      // Backend sends timestamp in seconds, convert to milliseconds for Date
      // TODO: Backend should send 'side' in WebSocket messages
      const restTrade: Trade = {
        id: msg.trade.id,
        market_id: msg.trade.market_id,
        buyer_address: msg.trade.buyer_address,
        seller_address: msg.trade.seller_address,
        buyer_order_id: msg.trade.buyer_order_id,
        seller_order_id: msg.trade.seller_order_id,
        price: msg.trade.price,
        size: msg.trade.size,
        side: 'buy' as const, // Default to buy, backend should send this
        timestamp: new Date(msg.trade.timestamp * 1000).toISOString(),
      };

      try {
        const enhanced = this.rest.enhanceTrade(restTrade);
        handler(enhanced); // Call the handler directly
      } catch (error) {
        console.error('[SDK] Failed to enhance trade:', error);
      }
    });

    // Subscribe to channel (WS handles queueing if not connected)
    this.ws.subscribe('trades', { marketId });

    // Return cleanup function
    return () => {
      console.log(`[SDK] Cleaning up trades subscription for ${marketId}`);
      removeHandler();
      this.ws.unsubscribe('trades', { marketId });
    };
  }

  /**
   * Stream orderbook updates for a market (enhanced with display values)
   * @returns Unsubscribe function
   */
  onOrderbook(marketId: string, handler: (update: { bids: EnhancedOrderbookLevel[], asks: EnhancedOrderbookLevel[] }) => void): () => void {
    console.log(`[SDK] onOrderbook called for ${marketId}`);

    // Register WebSocket message handler
    const removeHandler = this.ws.on('orderbook', (msg) => {
      if (msg.type !== 'orderbook' || msg.orderbook.market_id !== marketId) return;

      // Wait for cache if not ready yet
      if (!this.cacheInitialized) {
        console.warn('[SDK] Orderbook received before cache initialized, skipping');
        return;
      }

      try {
        // Enhance orderbook levels with display values
        const enhancedBids = msg.orderbook.bids.map(bid =>
          this.rest.enhanceOrderbookLevel(bid, marketId)
        );
        const enhancedAsks = msg.orderbook.asks.map(ask =>
          this.rest.enhanceOrderbookLevel(ask, marketId)
        );

        handler({ bids: enhancedBids, asks: enhancedAsks }); // Call the handler directly
      } catch (error) {
        console.error('[SDK] Failed to enhance orderbook:', error);
      }
    });

    // Subscribe to channel (WS handles queueing if not connected)
    this.ws.subscribe('orderbook', { marketId });

    // Return cleanup function
    return () => {
      console.log(`[SDK] Cleaning up orderbook subscription for ${marketId}`);
      removeHandler();
      this.ws.unsubscribe('orderbook', { marketId });
    };
  }

  /**
   * Stream order updates for a user
   * @returns Unsubscribe function
   */
  onUserOrders(userAddress: string, handler: (order: { order_id: string; status: string; filled_size: string }) => void): () => void {
    console.log(`[SDK] onUserOrders called for ${userAddress}`);

    // Register WebSocket message handler
    const removeHandler = this.ws.on('order', (msg) => {
      if (msg.type === 'order') {
        handler({ order_id: msg.order_id, status: msg.status, filled_size: msg.filled_size });
      }
    });

    // Subscribe to channel (WS handles queueing if not connected)
    this.ws.subscribe('user', { userAddress });

    // Return cleanup function
    return () => {
      console.log(`[SDK] Cleaning up user orders subscription for ${userAddress}`);
      removeHandler();
      this.ws.unsubscribe('user', { userAddress });
    };
  }

  /**
   * Stream trade updates for a user (enhanced with display values)
   * @returns Unsubscribe function
   */
  onUserTrades(userAddress: string, handler: (trade: EnhancedTrade) => void): () => void {
    console.log(`[SDK] onUserTrades called for ${userAddress}`);

    // Register WebSocket message handler
    const removeHandler = this.ws.on('trade', (msg) => {
      if (msg.type !== 'trade') return;

      // Wait for cache if not ready yet
      if (!this.cacheInitialized) {
        console.warn('[SDK] Trade received before cache initialized, skipping');
        return;
      }

      // Convert WebSocket TradeData to REST Trade format
      // Backend sends timestamp in seconds, convert to milliseconds for Date
      // TODO: Backend should send 'side' in WebSocket messages
      const restTrade: Trade = {
        id: msg.trade.id,
        market_id: msg.trade.market_id,
        buyer_address: msg.trade.buyer_address,
        seller_address: msg.trade.seller_address,
        buyer_order_id: msg.trade.buyer_order_id,
        seller_order_id: msg.trade.seller_order_id,
        price: msg.trade.price,
        size: msg.trade.size,
        side: 'buy' as const, // Default to buy, backend should send this
        timestamp: new Date(msg.trade.timestamp * 1000).toISOString(),
      };

      try {
        const enhanced = this.rest.enhanceTrade(restTrade);
        handler(enhanced); // Call the handler directly
      } catch (error) {
        console.error('[SDK] Failed to enhance user trade:', error);
      }
    });

    // Subscribe to channel (WS handles queueing if not connected)
    this.ws.subscribe('user', { userAddress });

    // Return cleanup function
    return () => {
      console.log(`[SDK] Cleaning up user trades subscription for ${userAddress}`);
      removeHandler();
      this.ws.unsubscribe('user', { userAddress });
    };
  }

  /**
   * Stream balance updates for a user
   * @returns Unsubscribe function
   */
  onUserBalances(userAddress: string, handler: (balance: { token_ticker: string; available: string; locked: string }) => void): () => void {
    console.log(`[SDK] onUserBalances called for ${userAddress}`);

    // Register WebSocket message handler
    const removeHandler = this.ws.on('balance', (msg) => {
      if (msg.type === 'balance') {
        handler({ token_ticker: msg.token_ticker, available: msg.available, locked: msg.locked });
      }
    });

    // Subscribe to channel (WS handles queueing if not connected)
    this.ws.subscribe('user', { userAddress });

    // Return cleanup function
    return () => {
      console.log(`[SDK] Cleaning up user balances subscription for ${userAddress}`);
      removeHandler();
      this.ws.unsubscribe('user', { userAddress });
    };
  }

  /**
   * Disconnect all connections
   */
  disconnect(): void {
    this.ws.disconnect();
  }
}
