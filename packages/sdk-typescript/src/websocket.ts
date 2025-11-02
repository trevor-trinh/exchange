/**
 * WebSocket client for real-time exchange data
 */

import type {
  ClientMessage,
  ServerMessage,
  SubscriptionChannel,
  MessageHandler,
} from './types/websocket';
import { WebSocketError } from './errors';

export interface WebSocketClientConfig {
  url: string;
  reconnectDelays?: number[];
  pingInterval?: number;
}

type MessageType = ServerMessage['type'];

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

  constructor(config: WebSocketClientConfig) {
    this.url = config.url;
    this.reconnectDelays = config.reconnectDelays ?? [1000, 2000, 4000, 8000, 16000];
    this.pingInterval = config.pingInterval ?? 30000;
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
          console.error('Failed to parse WebSocket message:', error);
        }
      };

      this.ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

      this.ws.onclose = () => {
        this.isConnected = false;
        this.stopPingTimer();
        this.scheduleReconnect();
      };
    } catch (error) {
      throw new WebSocketError('Failed to connect', error);
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
  subscribe(
    channel: SubscriptionChannel,
    params?: { marketId?: string; userAddress?: string }
  ): void {
    const key = this.getSubscriptionKey(channel, params);
    const currentCount = this.activeSubscriptions.get(key) || 0;

    // Only send subscribe message if this is the first subscription
    if (currentCount === 0) {
      console.log(`[WebSocket] Subscribing to ${key}`);
      const message: ClientMessage = {
        type: 'subscribe',
        channel,
        market_id: params?.marketId,
        user_address: params?.userAddress,
      };
      this.send(message);
    } else {
      console.log(`[WebSocket] Already subscribed to ${key} (count: ${currentCount}), incrementing ref count`);
    }

    // Increment reference count
    this.activeSubscriptions.set(key, currentCount + 1);
  }

  /**
   * Unsubscribe from a channel (with reference counting)
   */
  unsubscribe(
    channel: SubscriptionChannel,
    params?: { marketId?: string; userAddress?: string }
  ): void {
    const key = this.getSubscriptionKey(channel, params);
    const currentCount = this.activeSubscriptions.get(key) || 0;

    if (currentCount === 0) {
      console.warn(`[WebSocket] Attempted to unsubscribe from ${key} but no active subscription found`);
      return;
    }

    const newCount = currentCount - 1;

    // Only send unsubscribe message when ref count reaches 0
    if (newCount === 0) {
      console.log(`[WebSocket] Unsubscribing from ${key}`);
      const message: ClientMessage = {
        type: 'unsubscribe',
        channel,
        market_id: params?.marketId,
        user_address: params?.userAddress,
      };
      this.send(message);
      this.activeSubscriptions.delete(key);
    } else {
      console.log(`[WebSocket] Decrementing ref count for ${key} (count: ${newCount})`);
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
    const identifier = params?.marketId || params?.userAddress || 'global';
    return `${channel}:${identifier}`;
  }

  /**
   * Register a message handler
   */
  on<T extends MessageType>(
    type: T,
    handler: MessageHandler<Extract<ServerMessage, { type: T }>>
  ): () => void {
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
      console.error('Failed to send WebSocket message:', error);
      this.messageQueue.push(message);
    }
  }

  private handleMessage(message: ServerMessage): void {
    // Handle pong messages automatically
    if (message.type === 'pong') {
      this.lastPongTime = Date.now();
    }

    const handlers = this.handlers.get(message.type);
    if (handlers) {
      handlers.forEach((handler) => {
        try {
          handler(message);
        } catch (error) {
          console.error(`Error in ${message.type} handler:`, error);
        }
      });
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimeout) {
      return;
    }

    const delay = this.reconnectDelays[
      Math.min(this.reconnectAttempt, this.reconnectDelays.length - 1)
    ];

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
          console.warn('[WebSocket] No pong received, reconnecting...');
          this.ws?.close();
          return;
        }

        // Send ping
        this.send({ type: 'ping' });
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

    console.log(`[WebSocket] Re-subscribing to ${this.activeSubscriptions.size} subscriptions`);

    // Parse subscription keys and send subscribe messages
    this.activeSubscriptions.forEach((count, key) => {
      const [channel, identifier] = key.split(':') as [SubscriptionChannel, string];

      const message: ClientMessage = {
        type: 'subscribe',
        channel,
        // Determine if it's a market or user subscription
        market_id: channel === 'trades' || channel === 'orderbook' ? identifier : undefined,
        user_address: channel === 'user' ? identifier : undefined,
      };

      this.send(message);
      console.log(`[WebSocket] Re-subscribed to ${key}`);
    });
  }
}
