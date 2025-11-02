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
import type { RestClientConfig, Trade, Order, Balance, EnhancedTrade, EnhancedOrder, EnhancedBalance } from './rest';
import { WebSocketClient } from './websocket';
import type { WebSocketClientConfig } from './websocket';
import type { OrderbookLevel, TradeData } from './types/websocket';

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
  }

  /**
   * Initialize the SDK cache with markets and tokens
   * This is called automatically by WebSocket methods
   * @public - Can also be called manually for eager initialization
   */
  async initializeCache(): Promise<void> {
    // If already initialized, return immediately
    if (this.cacheInitialized) {
      console.log('[SDK] Cache already initialized, skipping');
      return;
    }

    // If initialization is in progress, wait for it
    if (this.cacheInitPromise) {
      console.log('[SDK] Cache initialization in progress, waiting...');
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
   * Automatically initializes cache if needed
   * @returns Unsubscribe function
   */
  onTrades(marketId: string, handler: (trade: EnhancedTrade) => void): () => void {
    console.log(`[SDK] onTrades called for ${marketId}`);

    // Initialize cache asynchronously before connecting
    this.initializeCache().then(() => {
      console.log(`[SDK] Cache ready, subscribing to trades for ${marketId}`);
      this.ws.connect();
      this.ws.subscribe('trades', { marketId });
    }).catch(error => {
      console.error('[SDK] Failed to initialize cache for trades:', error);
    });

    const removeHandler = this.ws.on('trade', (msg) => {
      if (msg.type === 'trade') {
        // Only process if cache is ready
        if (!this.cacheInitialized) {
          return; // Skip trades until cache is ready
        }

        // Convert WebSocket TradeData to REST Trade format
        const restTrade: Trade = {
          id: msg.trade.id,
          market_id: msg.trade.market_id,
          buyer_address: msg.trade.buyer_address,
          seller_address: msg.trade.seller_address,
          buyer_order_id: msg.trade.buyer_order_id,
          seller_order_id: msg.trade.seller_order_id,
          price: msg.trade.price,
          size: msg.trade.size,
          timestamp: new Date(msg.trade.timestamp).toISOString(),
        };

        // Enhance and pass to handler
        try {
          const enhanced = this.rest.enhanceTrade(restTrade);
          handler(enhanced);
        } catch (error) {
          console.error('[SDK] Failed to enhance trade:', error);
        }
      }
    });

    // Return cleanup function that both removes handler AND unsubscribes
    return () => {
      console.log(`[SDK] Cleaning up trades subscription for ${marketId}`);
      removeHandler(); // Remove message handler
      this.ws.unsubscribe('trades', { marketId }); // Unsubscribe from channel
    };
  }

  /**
   * Stream orderbook updates for a market
   * Automatically initializes cache if needed
   * @returns Unsubscribe function
   */
  onOrderbook(marketId: string, handler: (update: { bids: OrderbookLevel[], asks: OrderbookLevel[] }) => void): () => void {
    console.log(`[SDK] onOrderbook called for ${marketId}`);

    // Initialize cache asynchronously before connecting
    this.initializeCache().then(() => {
      console.log(`[SDK] Cache ready, subscribing to orderbook for ${marketId}`);
      this.ws.connect();
      this.ws.subscribe('orderbook', { marketId });
    }).catch(error => {
      console.error('[SDK] Failed to initialize cache for orderbook:', error);
    });

    const removeHandler = this.ws.on('orderbook', (msg) => {
      if (msg.type === 'orderbook') {
        handler({ bids: msg.orderbook.bids, asks: msg.orderbook.asks });
      }
    });

    // Return cleanup function that both removes handler AND unsubscribes
    return () => {
      console.log(`[SDK] Cleaning up orderbook subscription for ${marketId}`);
      removeHandler(); // Remove message handler
      this.ws.unsubscribe('orderbook', { marketId }); // Unsubscribe from channel
    };
  }

  /**
   * Stream order updates for a user
   * @returns Unsubscribe function
   */
  onUserOrders(userAddress: string, handler: (order: { order_id: string; status: string; filled_size: string }) => void): () => void {
    console.log(`[SDK] onUserOrders called for ${userAddress}`);
    this.ws.connect();
    this.ws.subscribe('user', { userAddress });

    const removeHandler = this.ws.on('order', (msg) => {
      if (msg.type === 'order') {
        handler({ order_id: msg.order_id, status: msg.status, filled_size: msg.filled_size });
      }
    });

    // Return cleanup function
    return () => {
      console.log(`[SDK] Cleaning up user orders subscription for ${userAddress}`);
      removeHandler();
      this.ws.unsubscribe('user', { userAddress });
    };
  }

  /**
   * Stream trade updates for a user (enhanced with display values)
   * Automatically initializes cache if needed
   * @returns Unsubscribe function
   */
  onUserTrades(userAddress: string, handler: (trade: EnhancedTrade) => void): () => void {
    console.log(`[SDK] onUserTrades called for ${userAddress}`);

    // Initialize cache asynchronously before connecting
    this.initializeCache().then(() => {
      console.log(`[SDK] Cache ready, subscribing to user trades for ${userAddress}`);
      this.ws.connect();
      this.ws.subscribe('user', { userAddress });
    }).catch(error => {
      console.error('[SDK] Failed to initialize cache for user trades:', error);
    });

    const removeHandler = this.ws.on('trade', (msg) => {
      if (msg.type === 'trade') {
        // Only process if cache is ready
        if (!this.cacheInitialized) {
          return; // Skip trades until cache is ready
        }

        // Convert WebSocket TradeData to REST Trade format
        const restTrade: Trade = {
          id: msg.trade.id,
          market_id: msg.trade.market_id,
          buyer_address: msg.trade.buyer_address,
          seller_address: msg.trade.seller_address,
          buyer_order_id: msg.trade.buyer_order_id,
          seller_order_id: msg.trade.seller_order_id,
          price: msg.trade.price,
          size: msg.trade.size,
          timestamp: new Date(msg.trade.timestamp).toISOString(),
        };

        try {
          const enhanced = this.rest.enhanceTrade(restTrade);
          handler(enhanced);
        } catch (error) {
          console.error('[SDK] Failed to enhance user trade:', error);
        }
      }
    });

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
    this.ws.connect();
    this.ws.subscribe('user', { userAddress });

    const removeHandler = this.ws.on('balance', (msg) => {
      if (msg.type === 'balance') {
        handler({ token_ticker: msg.token_ticker, available: msg.available, locked: msg.locked });
      }
    });

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
