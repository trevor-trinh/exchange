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

import { RestClient } from "./rest";
import type {
  Trade,
  Order,
  Balance,
  EnhancedTrade,
  EnhancedOrder,
  EnhancedBalance,
  EnhancedOrderbookLevel,
} from "./rest";
import { WebSocketClient } from "./websocket";
import type { OrderbookLevel, TradeData, ServerMessage } from "./types/websocket";
import { CacheService } from "./cache";
import { EnhancementService } from "./enhancement";
import { ConsoleLogger, NoopLogger } from "./logger";
import type { Logger, LogLevel } from "./logger";

export { RestClient } from "./rest";
export { WebSocketClient } from "./websocket";
export { CacheService } from "./cache";
export { EnhancementService } from "./enhancement";
export { ConsoleLogger, NoopLogger } from "./logger";

export { SdkError, ApiError, WebSocketError, ValidationError } from "./errors";

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
} from "./rest";

export type {
  ClientMessage,
  ServerMessage,
  SubscriptionChannel,
  MessageHandler,
  OrderbookLevel,
  TradeData,
  OrderbookData,
} from "./types/websocket";

export type { Logger, LogLevel } from "./logger";

// Re-export generated types for advanced usage
export type { components } from "./types/generated";

/**
 * Main Exchange SDK client
 */
export interface ExchangeClientConfig {
  restUrl: string;
  wsUrl: string;
  restTimeout?: number;
  wsReconnectDelays?: number[];
  wsPingInterval?: number;
  logLevel?: LogLevel;
  logger?: Logger;
}

export class ExchangeClient {
  public readonly rest: RestClient;
  public readonly ws: WebSocketClient;
  public readonly cache: CacheService;
  public readonly enhancer: EnhancementService;
  public readonly logger: Logger;

  private cacheInitPromise: Promise<void> | null = null;

  constructor(config: string | ExchangeClientConfig) {
    // Support both simple URL string and full config object
    const cfg =
      typeof config === "string"
        ? {
            restUrl: config,
            wsUrl: config.replace(/^http/, "ws") + "/ws",
          }
        : config;

    // Create shared services
    this.logger = cfg.logger ?? new ConsoleLogger({ level: cfg.logLevel ?? "info" });
    this.cache = new CacheService(this.logger);
    this.enhancer = new EnhancementService(this.cache);

    // Create clients with shared services
    this.rest = new RestClient({
      baseUrl: cfg.restUrl,
      timeout: cfg.restTimeout,
      cache: this.cache,
      enhancer: this.enhancer,
      logger: this.logger,
    });

    this.ws = new WebSocketClient({
      url: cfg.wsUrl,
      reconnectDelays: cfg.wsReconnectDelays,
      pingInterval: cfg.wsPingInterval,
      cache: this.cache,
      enhancer: this.enhancer,
      logger: this.logger,
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
    this.initializeCache().catch((error) => {
      this.logger.error("Failed to initialize cache", error);
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
    if (this.cache.isReady()) {
      return;
    }

    // If initialization is in progress, wait for it
    if (this.cacheInitPromise) {
      return this.cacheInitPromise;
    }

    this.logger.info("Starting cache initialization...");

    // Start initialization
    this.cacheInitPromise = (async () => {
      try {
        await Promise.all([this.rest.getMarkets(), this.rest.getTokens()]);
        this.cache.markInitialized();
      } catch (error) {
        this.logger.error("Failed to initialize cache", error);
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
  getCandles(params: { marketId: string; interval: string; from: number; to: number }) {
    return this.rest.getCandles(params);
  }

  /**
   * Place an order
   */
  placeOrder(params: {
    userAddress: string;
    marketId: string;
    side: "buy" | "sell";
    orderType: "limit" | "market";
    price: string;
    size: string;
    signature: string;
  }) {
    return this.rest.placeOrder(params);
  }

  /**
   * Cancel an order
   */
  cancelOrder(params: { userAddress: string; orderId: string; signature: string }) {
    return this.rest.cancelOrder(params);
  }

  /**
   * Cancel all orders for a user, optionally filtered by market
   */
  cancelAllOrders(params: { userAddress: string; marketId?: string; signature: string }) {
    return this.rest.cancelAllOrders(params);
  }

  // ============================================================================
  // Convenience Methods - WebSocket (delegate to ws)
  // ============================================================================

  /**
   * Stream trades for a market
   */
  onTrades(marketId: string, handler: (trade: EnhancedTrade) => void) {
    return this.ws.onTrades(marketId, handler);
  }

  /**
   * Stream orderbook updates for a market
   */
  onOrderbook(
    marketId: string,
    handler: (update: { bids: EnhancedOrderbookLevel[]; asks: EnhancedOrderbookLevel[] }) => void
  ) {
    return this.ws.onOrderbook(marketId, handler);
  }

  /**
   * Stream order updates for a user
   */
  onUserOrders(
    userAddress: string,
    handler: (order: { order_id: string; status: string; filled_size: string }) => void
  ) {
    return this.ws.onUserOrders(userAddress, handler);
  }

  /**
   * Stream trade updates for a user
   */
  onUserTrades(userAddress: string, handler: (trade: EnhancedTrade) => void) {
    return this.ws.onUserTrades(userAddress, handler);
  }

  /**
   * Stream balance updates for a user
   */
  onUserBalances(
    userAddress: string,
    handler: (balance: { token_ticker: string; available: string; locked: string }) => void
  ) {
    return this.ws.onUserBalances(userAddress, handler);
  }

  /**
   * Disconnect all connections
   */
  disconnect(): void {
    this.ws.disconnect();
  }
}
