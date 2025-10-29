/**
 * Main library exports
 */

// API
export { getAPI, APIError } from './api';
export type { ExchangeAPI } from './api';

// WebSocket
export { getWebSocketManager, resetWebSocketManager } from './websocket';
export type { WebSocketManager } from './websocket';

// Store
export { useExchangeStore } from './store';
export {
  selectSelectedMarket,
  selectOrderbookBids,
  selectOrderbookAsks,
  selectRecentTrades,
  selectPriceHistory,
  selectCurrentPrice,
} from './store';

// Hooks
export * from './hooks';

// Types
export type * from './types/exchange';
export type * from './types/websocket';
