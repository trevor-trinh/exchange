/**
 * WebSocket manager for real-time exchange data
 * Now using the @exchange/sdk package
 */

import { WebSocketClient } from "@exchange/sdk";
import type { ServerMessage, SubscriptionChannel } from "@exchange/sdk";

type MessageHandler = (message: ServerMessage) => void;
type MessageType = ServerMessage["type"];

export class WebSocketManager {
  private client: WebSocketClient;
  private handlers = new Map<MessageType, Set<MessageHandler>>();

  constructor(url: string) {
    this.client = new WebSocketClient({ url });
    this.setupMessageHandlers();
    this.client.connect();
  }

  /**
   * Setup message handlers to forward SDK messages to local handlers
   */
  private setupMessageHandlers(): void {
    // Forward all message types from SDK client to local handlers
    const messageTypes: MessageType[] = [
      "subscribed",
      "unsubscribed",
      "trade",
      "orderbook_snapshot",
      "orderbook_update",
      "order_placed",
      "order_cancelled",
      "pong",
      "error",
    ];

    messageTypes.forEach((type) => {
      this.client.on(type, (message) => {
        const handlers = this.handlers.get(type);
        if (handlers) {
          handlers.forEach((handler) => {
            try {
              handler(message);
            } catch (error) {
              console.error("[WebSocket] Handler error:", error);
            }
          });
        }
      });
    });
  }

  /**
   * Register a message handler
   */
  on(type: MessageType, handler: MessageHandler): void {
    if (!this.handlers.has(type)) {
      this.handlers.set(type, new Set());
    }
    this.handlers.get(type)!.add(handler);
  }

  /**
   * Unregister a message handler
   */
  off(type: MessageType, handler: MessageHandler): void {
    const handlers = this.handlers.get(type);
    if (handlers) {
      handlers.delete(handler);
      if (handlers.size === 0) {
        this.handlers.delete(type);
      }
    }
  }

  /**
   * Subscribe to a channel
   */
  subscribe(channel: SubscriptionChannel, marketId?: string, userAddress?: string): void {
    this.client.subscribe(channel, { marketId, userAddress });
  }

  /**
   * Unsubscribe from a channel
   */
  unsubscribe(channel: SubscriptionChannel, marketId?: string, userAddress?: string): void {
    this.client.unsubscribe(channel, { marketId, userAddress });
  }

  /**
   * Check if connected
   */
  isReady(): boolean {
    return this.client.isReady();
  }

  /**
   * Close connection and clean up
   */
  close(): void {
    console.log("[WebSocket] Closing connection");
    this.client.disconnect();
    this.handlers.clear();
  }
}

// Singleton instance
let wsManagerInstance: WebSocketManager | null = null;

export function getWebSocketManager(): WebSocketManager {
  if (!wsManagerInstance) {
    const wsUrl = process.env.NEXT_PUBLIC_WS_URL || "ws://localhost:8888/ws";
    wsManagerInstance = new WebSocketManager(wsUrl);
  }
  return wsManagerInstance;
}

// Helper to reset for testing
export function resetWebSocketManager(): void {
  if (wsManagerInstance) {
    wsManagerInstance.close();
    wsManagerInstance = null;
  }
}
