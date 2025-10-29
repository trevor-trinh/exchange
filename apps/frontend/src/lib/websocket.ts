/**
 * WebSocket manager for real-time exchange data
 *
 * Features:
 * - Automatic reconnection with exponential backoff
 * - Message queueing when disconnected
 * - Type-safe message handlers
 * - Handler registration/cleanup
 */

import type { ClientMessage, ServerMessage } from './types/websocket';

type MessageHandler = (message: ServerMessage) => void;
type MessageType = ServerMessage['type'];

export class WebSocketManager {
  private ws: WebSocket | null = null;
  private isConnected = false;
  private messageQueue: ClientMessage[] = [];
  private handlers = new Map<MessageType, Set<MessageHandler>>();
  private reconnectAttempts = 0;
  private reconnectTimeout: NodeJS.Timeout | null = null;
  private url: string;

  // Reconnection config
  private readonly maxReconnectAttempts = 10;
  private readonly reconnectDelays = [1000, 2000, 4000, 8000, 16000]; // ms

  constructor(url: string) {
    this.url = url;
    this.connect();
  }

  /**
   * Establish WebSocket connection
   */
  private connect(): void {
    if (this.ws?.readyState === WebSocket.OPEN || this.reconnectTimeout) {
      return;
    }

    try {
      console.log('[WebSocket] Connecting to', this.url);
      this.ws = new WebSocket(this.url);

      this.ws.onopen = this.handleOpen.bind(this);
      this.ws.onclose = this.handleClose.bind(this);
      this.ws.onerror = this.handleError.bind(this);
      this.ws.onmessage = this.handleMessage.bind(this);
    } catch (error) {
      console.error('[WebSocket] Connection error:', error);
      this.scheduleReconnect();
    }
  }

  /**
   * Handle connection open
   */
  private handleOpen(): void {
    console.log('[WebSocket] Connected');
    this.isConnected = true;
    this.reconnectAttempts = 0;
    this.processMessageQueue();
  }

  /**
   * Handle connection close
   */
  private handleClose(): void {
    console.log('[WebSocket] Disconnected');
    this.isConnected = false;
    this.cleanup();
    this.scheduleReconnect();
  }

  /**
   * Handle connection error
   */
  private handleError(event: Event): void {
    console.error('[WebSocket] Error:', event);
    this.cleanup();
    this.scheduleReconnect();
  }

  /**
   * Handle incoming message
   */
  private handleMessage(event: MessageEvent): void {
    try {
      const message: ServerMessage = JSON.parse(event.data);

      if (process.env.NODE_ENV === 'development') {
        console.log('[WebSocket] Received:', message.type, message);
      }

      // Call all handlers registered for this message type
      const handlers = this.handlers.get(message.type);
      if (handlers) {
        handlers.forEach((handler) => {
          try {
            handler(message);
          } catch (error) {
            console.error('[WebSocket] Handler error:', error);
          }
        });
      }
    } catch (error) {
      console.error('[WebSocket] Failed to parse message:', error);
    }
  }

  /**
   * Schedule reconnection with exponential backoff
   */
  private scheduleReconnect(): void {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
    }

    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('[WebSocket] Max reconnection attempts reached');
      return;
    }

    const delayIndex = Math.min(
      this.reconnectAttempts,
      this.reconnectDelays.length - 1
    );
    const delay = this.reconnectDelays[delayIndex];

    console.log(
      `[WebSocket] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts + 1}/${this.maxReconnectAttempts})`
    );

    this.reconnectTimeout = setTimeout(() => {
      this.reconnectTimeout = null;
      this.reconnectAttempts++;
      this.connect();
    }, delay);
  }

  /**
   * Clean up WebSocket resources
   */
  private cleanup(): void {
    if (this.reconnectTimeout) {
      clearTimeout(this.reconnectTimeout);
      this.reconnectTimeout = null;
    }

    if (this.ws) {
      this.ws.onopen = null;
      this.ws.onclose = null;
      this.ws.onerror = null;
      this.ws.onmessage = null;

      if (this.ws.readyState === WebSocket.OPEN) {
        this.ws.close();
      }
      this.ws = null;
    }
  }

  /**
   * Process queued messages
   */
  private processMessageQueue(): void {
    while (this.messageQueue.length > 0) {
      const message = this.messageQueue.shift();
      if (message) {
        this.send(message);
      }
    }
  }

  /**
   * Send message to server
   * Queues message if not connected
   */
  send(message: ClientMessage): void {
    if (!this.isConnected || !this.ws || this.ws.readyState !== WebSocket.OPEN) {
      this.messageQueue.push(message);
      return;
    }

    if (process.env.NODE_ENV === 'development') {
      console.log('[WebSocket] Sending:', message.type, message);
    }

    this.ws.send(JSON.stringify(message));
  }

  /**
   * Register a message handler
   */
  on(type: MessageType, handler: MessageHandler): void {
    if (!this.handlers.has(type)) {
      this.handlers.set(type, new Set());
    }
    this.handlers.get(type)!.add(handler);

    if (process.env.NODE_ENV === 'development') {
      console.log(`[WebSocket] Handler registered for: ${type}`);
    }
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
  subscribe(
    channel: 'Trades' | 'Orderbook' | 'User',
    marketId?: string,
    userAddress?: string
  ): void {
    this.send({
      type: 'subscribe',
      channel,
      market_id: marketId,
      user_address: userAddress,
    });
  }

  /**
   * Unsubscribe from a channel
   */
  unsubscribe(
    channel: 'Trades' | 'Orderbook' | 'User',
    marketId?: string,
    userAddress?: string
  ): void {
    this.send({
      type: 'unsubscribe',
      channel,
      market_id: marketId,
      user_address: userAddress,
    });
  }

  /**
   * Send ping
   */
  ping(): void {
    this.send({ type: 'ping' });
  }

  /**
   * Close connection and clean up
   */
  close(): void {
    console.log('[WebSocket] Closing connection');
    this.cleanup();
    this.handlers.clear();
    this.messageQueue = [];
  }
}

// Singleton instance
let wsManagerInstance: WebSocketManager | null = null;

export function getWebSocketManager(): WebSocketManager {
  if (!wsManagerInstance) {
    const wsUrl = process.env.NEXT_PUBLIC_WS_URL || 'ws://localhost:8888/ws';
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
