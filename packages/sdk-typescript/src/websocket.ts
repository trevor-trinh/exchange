/**
 * WebSocket client for real-time exchange data
 */

import type { ClientMessage, ServerMessage, SubscriptionChannel, MessageHandler } from "./types/websocket";
import { WebSocketError } from "./errors";
import type { EnhancedTrade, EnhancedOrderbookLevel, WsTradeData } from "./enhancement";
import type { CacheService } from "./cache";
import type { EnhancementService } from "./enhancement";
import type { Logger } from "./logger";

export interface WebSocketClientConfig {
  url: string;
  reconnectDelays?: number[];
  pingInterval?: number;
  cache: CacheService;
  enhancer: EnhancementService;
  logger: Logger;
}

type MessageType = ServerMessage["type"];

// Subscription key for tracking active subscriptions
type SubscriptionKey = string;

export class WebSocketClient {
  private ws: WebSocket | null = null;
  private url: string;
  private isConnected = false;
  private reconnectDelays: number[];
  private reconnectAttempt = 0;
  private reconnectTimeout: ReturnType<typeof setTimeout> | null = null;
  private pingInterval: number;
  private pingTimer: ReturnType<typeof setInterval> | null = null;
  private lastPongTime: number = Date.now();
  private pongTimeout: number = 60000; // 60 seconds

  private messageQueue: ClientMessage[] = [];
  private handlers = new Map<MessageType, Set<MessageHandler>>();

  // Track active subscriptions to prevent duplicates
  // Key format: "channel:marketId" or "channel:userAddress"
  private activeSubscriptions = new Map<SubscriptionKey, number>(); // ref count

  private cache: CacheService;
  private enhancer: EnhancementService;
  private logger: Logger;

  constructor(config: WebSocketClientConfig) {
    this.url = config.url;
    this.reconnectDelays = config.reconnectDelays ?? [1000, 2000, 4000, 8000, 16000];
    this.pingInterval = config.pingInterval ?? 30000;
    this.cache = config.cache;
    this.enhancer = config.enhancer;
    this.logger = config.logger;
  }

  /**
   * Connect to the WebSocket server
   */
  connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      return;
    }

    try {
      this.ws = new WebSocket(this.url);

      this.ws.onopen = () => {
        this.isConnected = true;
        this.reconnectAttempt = 0;
        this.lastPongTime = Date.now();

        // Re-subscribe to all active subscriptions after reconnect
        this.resubscribeAll();

        // Send queued messages
        while (this.messageQueue.length > 0) {
          const msg = this.messageQueue.shift()!;
          this.send(msg);
        }

        // Start ping timer
        this.startPingTimer();
      };

      this.ws.onmessage = (event) => {
        try {
          const message: ServerMessage = JSON.parse(event.data);
          this.handleMessage(message);
        } catch (error) {
          this.logger.error("Failed to parse WebSocket message", error);
        }
      };

      this.ws.onerror = (error) => {
        this.logger.error("WebSocket error", error);
      };

      this.ws.onclose = () => {
        this.isConnected = false;
        this.stopPingTimer();
        this.scheduleReconnect();
      };
    } catch (error) {
      throw new WebSocketError("Failed to connect", error);
    }
  }

  /**
   * Disconnect from the WebSocket server
   */
  disconnect(): void {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    this.stopPingTimer();

    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }

    this.isConnected = false;
    this.messageQueue = [];
  }

  /**
   * Subscribe to a channel (with reference counting to prevent duplicates)
   */
  subscribe(channel: SubscriptionChannel, params?: { marketId?: string; userAddress?: string }): void {
    const key = this.getSubscriptionKey(channel, params);
    const currentCount = this.activeSubscriptions.get(key) || 0;

    // Only send subscribe message if this is the first subscription
    if (currentCount === 0) {
      this.logger.debug(`Subscribing to ${key}`);
      const message: ClientMessage = {
        type: "subscribe",
        channel,
        ...(params?.marketId && { market_id: params.marketId }),
        ...(params?.userAddress && { user_address: params.userAddress }),
      };
      this.send(message);
    } else {
      this.logger.debug(`Already subscribed to ${key} (count: ${currentCount}), incrementing ref count`);
    }

    // Increment reference count
    this.activeSubscriptions.set(key, currentCount + 1);
  }

  /**
   * Unsubscribe from a channel (with reference counting)
   */
  unsubscribe(channel: SubscriptionChannel, params?: { marketId?: string; userAddress?: string }): void {
    const key = this.getSubscriptionKey(channel, params);
    const currentCount = this.activeSubscriptions.get(key) || 0;

    if (currentCount === 0) {
      this.logger.warn(`Attempted to unsubscribe from ${key} but no active subscription found`);
      return;
    }

    const newCount = currentCount - 1;

    // Only send unsubscribe message when ref count reaches 0
    if (newCount === 0) {
      this.logger.debug(`Unsubscribing from ${key}`);
      const message: ClientMessage = {
        type: "unsubscribe",
        channel,
        ...(params?.marketId && { market_id: params.marketId }),
        ...(params?.userAddress && { user_address: params.userAddress }),
      };
      this.send(message);
      this.activeSubscriptions.delete(key);
    } else {
      this.logger.debug(`Decrementing ref count for ${key} (count: ${newCount})`);
      this.activeSubscriptions.set(key, newCount);
    }
  }

  /**
   * Get subscription key for tracking
   */
  private getSubscriptionKey(
    channel: SubscriptionChannel,
    params?: { marketId?: string; userAddress?: string }
  ): SubscriptionKey {
    const identifier = params?.marketId || params?.userAddress || "global";
    return `${channel}:${identifier}`;
  }

  /**
   * Register a message handler
   */
  on<T extends MessageType>(type: T, handler: MessageHandler<Extract<ServerMessage, { type: T }>>): () => void {
    if (!this.handlers.has(type)) {
      this.handlers.set(type, new Set());
    }
    this.handlers.get(type)!.add(handler as MessageHandler);

    // Return unsubscribe function
    return () => {
      const handlers = this.handlers.get(type);
      if (handlers) {
        handlers.delete(handler as MessageHandler);
      }
    };
  }

  /**
   * Remove a message handler
   */
  off(type: MessageType, handler: MessageHandler): void {
    const handlers = this.handlers.get(type);
    if (handlers) {
      handlers.delete(handler);
    }
  }

  /**
   * Remove all handlers for a message type
   */
  removeAllHandlers(type?: MessageType): void {
    if (type) {
      this.handlers.delete(type);
    } else {
      this.handlers.clear();
    }
  }

  /**
   * Check if connected
   */
  isReady(): boolean {
    return this.isConnected && this.ws?.readyState === WebSocket.OPEN;
  }

  // ===== Private Methods =====

  private send(message: ClientMessage): void {
    if (!this.isConnected || this.ws?.readyState !== WebSocket.OPEN) {
      this.messageQueue.push(message);
      return;
    }

    try {
      this.ws.send(JSON.stringify(message));
    } catch (error) {
      this.logger.error("Failed to send WebSocket message", error);
      this.messageQueue.push(message);
    }
  }

  private handleMessage(message: ServerMessage): void {
    // Handle pong messages automatically
    if (message.type === "pong") {
      this.lastPongTime = Date.now();
    }

    const handlers = this.handlers.get(message.type);
    if (handlers) {
      handlers.forEach((handler) => {
        try {
          handler(message);
        } catch (error) {
          this.logger.error(`Error in ${message.type} handler`, error);
        }
      });
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimeout) {
      return;
    }

    const delay = this.reconnectDelays[Math.min(this.reconnectAttempt, this.reconnectDelays.length - 1)];

    this.reconnectTimeout = setTimeout(() => {
      this.reconnectTimeout = null;
      this.reconnectAttempt++;
      this.connect();
    }, delay);
  }

  private startPingTimer(): void {
    this.stopPingTimer();
    this.pingTimer = setInterval(() => {
      if (this.isConnected) {
        // Check if we've received a pong recently
        const timeSinceLastPong = Date.now() - this.lastPongTime;
        if (timeSinceLastPong > this.pongTimeout) {
          this.logger.warn("No pong received, reconnecting...");
          this.ws?.close();
          return;
        }

        // Send ping
        this.send({ type: "ping" });
      }
    }, this.pingInterval);
  }

  private stopPingTimer(): void {
    if (this.pingTimer) {
      clearInterval(this.pingTimer);
      this.pingTimer = null;
    }
  }

  /**
   * Re-subscribe to all active subscriptions (used after reconnect)
   */
  private resubscribeAll(): void {
    if (this.activeSubscriptions.size === 0) {
      return;
    }

    this.logger.info(`Re-subscribing to ${this.activeSubscriptions.size} subscriptions`);

    // Parse subscription keys and send subscribe messages
    this.activeSubscriptions.forEach((count, key) => {
      const [channel, identifier] = key.split(":") as [SubscriptionChannel, string];

      const message: ClientMessage = {
        type: "subscribe",
        channel,
        // Determine if it's a market or user subscription - only include the relevant field
        ...(channel === "trades" || channel === "orderbook" ? { market_id: identifier } : {}),
        ...(channel === "user_fills" || channel === "user_orders" || channel === "user_balances"
          ? { user_address: identifier }
          : {}),
      };

      this.send(message);
      this.logger.debug(`Re-subscribed to ${key}`);
    });
  }

  // ============================================================================
  // Type-Safe Convenience Methods (WebSocket-specific logic)
  // ============================================================================

  /**
   * Stream trades for a market (enhanced with display values)
   * @returns Unsubscribe function
   */
  onTrades(marketId: string, handler: (trade: EnhancedTrade) => void): () => void {
    this.logger.debug(`Setting up trades subscription for ${marketId}`);

    const removeHandler = this.on("trade", (msg) => {
      if (msg.type !== "trade" || msg.trade.market_id !== marketId) return;

      if (!this.cache.isReady()) {
        this.logger.warn("Trade received before cache initialized, skipping");
        return;
      }

      try {
        const wsTrade: WsTradeData = {
          id: msg.trade.id,
          market_id: msg.trade.market_id,
          buyer_address: msg.trade.buyer_address,
          seller_address: msg.trade.seller_address,
          buyer_order_id: msg.trade.buyer_order_id,
          seller_order_id: msg.trade.seller_order_id,
          price: msg.trade.price,
          size: msg.trade.size,
          side: msg.trade.side,
          timestamp: msg.trade.timestamp,
        };

        const enhanced = this.enhancer.enhanceWsTrade(wsTrade);
        handler(enhanced);
      } catch (error) {
        this.logger.error("Failed to enhance trade", error);
      }
    });

    this.subscribe("trades", { marketId });

    return () => {
      this.logger.debug(`Cleaning up trades subscription for ${marketId}`);
      removeHandler();
      this.unsubscribe("trades", { marketId });
    };
  }

  /**
   * Stream orderbook updates for a market (enhanced with display values)
   * @returns Unsubscribe function
   */
  onOrderbook(
    marketId: string,
    handler: (update: { bids: EnhancedOrderbookLevel[]; asks: EnhancedOrderbookLevel[] }) => void
  ): () => void {
    this.logger.debug(`Setting up orderbook subscription for ${marketId}`);

    const removeHandler = this.on("orderbook", (msg) => {
      if (msg.type !== "orderbook" || msg.orderbook.market_id !== marketId) return;

      if (!this.cache.isReady()) {
        this.logger.warn("Orderbook received before cache initialized, skipping");
        return;
      }

      try {
        const enhancedBids = msg.orderbook.bids.map((bid) => this.enhancer.enhanceOrderbookLevel(bid, marketId));
        const enhancedAsks = msg.orderbook.asks.map((ask) => this.enhancer.enhanceOrderbookLevel(ask, marketId));

        handler({ bids: enhancedBids, asks: enhancedAsks });
      } catch (error) {
        this.logger.error("Failed to enhance orderbook", error);
      }
    });

    this.subscribe("orderbook", { marketId });

    return () => {
      this.logger.debug(`Cleaning up orderbook subscription for ${marketId}`);
      removeHandler();
      this.unsubscribe("orderbook", { marketId });
    };
  }

  /**
   * Stream order updates for a user
   * @returns Unsubscribe function
   */
  onUserOrders(
    userAddress: string,
    handler: (order: { order_id: string; status: string; filled_size: string }) => void
  ): () => void {
    this.logger.debug(`Setting up user orders subscription for ${userAddress}`);

    const removeHandler = this.on("user_order", (msg) => {
      if (msg.type === "user_order") {
        handler({ order_id: msg.order_id, status: msg.status, filled_size: msg.filled_size });
      }
    });

    this.subscribe("user_orders", { userAddress });

    return () => {
      this.logger.debug(`Cleaning up user orders subscription for ${userAddress}`);
      removeHandler();
      this.unsubscribe("user_orders", { userAddress });
    };
  }

  /**
   * Stream trade updates for a user (enhanced with display values)
   * @returns Unsubscribe function
   */
  onUserFills(userAddress: string, handler: (trade: EnhancedTrade) => void): () => void {
    this.logger.debug(`Setting up user fills subscription for ${userAddress}`);

    const removeHandler = this.on("user_fill", (msg) => {
      if (msg.type !== "user_fill") return;

      if (!this.cache.isReady()) {
        this.logger.warn("Trade received before cache initialized, skipping");
        return;
      }

      try {
        const wsTrade: WsTradeData = {
          id: msg.trade.id,
          market_id: msg.trade.market_id,
          buyer_address: msg.trade.buyer_address,
          seller_address: msg.trade.seller_address,
          buyer_order_id: msg.trade.buyer_order_id,
          seller_order_id: msg.trade.seller_order_id,
          price: msg.trade.price,
          size: msg.trade.size,
          side: msg.trade.side,
          timestamp: msg.trade.timestamp,
        };

        const enhanced = this.enhancer.enhanceWsTrade(wsTrade);
        handler(enhanced);
      } catch (error) {
        this.logger.error("Failed to enhance user trade", error);
      }
    });

    this.subscribe("user_fills", { userAddress });

    return () => {
      this.logger.debug(`Cleaning up user fills subscription for ${userAddress}`);
      removeHandler();
      this.unsubscribe("user_fills", { userAddress });
    };
  }

  /**
   * Stream balance updates for a user
   * @returns Unsubscribe function
   */
  onUserBalances(
    userAddress: string,
    handler: (balance: { token_ticker: string; available: string; locked: string }) => void
  ): () => void {
    this.logger.debug(`Setting up user balances subscription for ${userAddress}`);

    const removeHandler = this.on("user_balance", (msg) => {
      if (msg.type === "user_balance") {
        handler({ token_ticker: msg.token_ticker, available: msg.available, locked: msg.locked });
      }
    });

    this.subscribe("user_balances", { userAddress });

    return () => {
      this.logger.debug(`Cleaning up user balances subscription for ${userAddress}`);
      removeHandler();
      this.unsubscribe("user_balances", { userAddress });
    };
  }
}
