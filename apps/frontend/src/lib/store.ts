/**
 * Global state management with Zustand
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";
import type { Market, Token, Orderbook, Trade, OrderbookLevel, Balance, Order, OrderStatus } from "./types/exchange";

// ============================================================================
// State Interface
// ============================================================================

interface ExchangeState {
  // Market data (Records for O(1) lookup/insert/update)
  markets: Record<string, Market>;
  tokens: Record<string, Token>;

  // UI Data
  selectedMarketId: string | null;
  orderbook: Orderbook | null;
  recentTrades: Trade[]; // Keep as array for chronological ordering
  selectedPrice: number | null;

  // User Data
  userAddress: string | null;
  isAuthenticated: boolean;
  userBalances: Record<string, Balance>; // Keyed by token_ticker
  userOrders: Record<string, Order>; // Keyed by order id
  userTrades: Trade[]; // Keep as array for chronological ordering

  // Actions - Market Data
  setMarkets: (markets: Market[]) => void;
  setTokens: (tokens: Token[]) => void;

  // Actions - UI Data
  selectMarket: (marketId: string) => void;
  setSelectedPrice: (price: number | null) => void;
  updateOrderbook: (marketId: string, bids: OrderbookLevel[], asks: OrderbookLevel[]) => void;
  addTrade: (trade: Trade) => void;

  // Actions - User Data
  setUser: (address: string) => void;
  clearUser: () => void;
  setBalances: (balances: Balance[]) => void;
  updateBalance: (tokenTicker: string, available: string, locked: string) => void;
  setOrders: (orders: Order[]) => void;
  updateOrder: (orderId: string, status: OrderStatus, filledSize: string) => void;
  setUserTrades: (trades: Trade[]) => void;
  addUserTrade: (trade: Trade) => void;

  // Utilities
  reset: () => void;
}

// ============================================================================
// Initial State
// ============================================================================

const initialState = {
  // Market Data
  markets: {} as Record<string, Market>,
  tokens: {} as Record<string, Token>,

  // UI Data
  selectedMarketId: null,
  orderbook: null,
  recentTrades: [],
  selectedPrice: null,

  // User Data
  userAddress: null,
  isAuthenticated: false,
  userBalances: {} as Record<string, Balance>,
  userOrders: {} as Record<string, Order>,
  userTrades: [],
};

// ============================================================================
// Store
// ============================================================================

export const useExchangeStore = create<ExchangeState>()(
  devtools(
    immer((set) => ({
      ...initialState,

      // ========================================================================
      // Market Data Actions
      // ========================================================================

      setMarkets: (markets) =>
        set((state) => {
          // Convert array to Record keyed by id
          state.markets = markets.reduce(
            (acc, market) => {
              acc[market.id] = market;
              return acc;
            },
            {} as Record<string, Market>
          );
        }),

      setTokens: (tokens) =>
        set((state) => {
          // Convert array to Record keyed by ticker
          state.tokens = tokens.reduce(
            (acc, token) => {
              acc[token.ticker] = token;
              return acc;
            },
            {} as Record<string, Token>
          );
        }),

      // ========================================================================
      // UI Data Actions
      // ========================================================================

      selectMarket: (marketId) =>
        set((state) => {
          state.selectedMarketId = marketId;
          state.orderbook = null;
          state.recentTrades = [];
        }),

      setSelectedPrice: (price) =>
        set((state) => {
          state.selectedPrice = price;
        }),

      updateOrderbook: (marketId, bids, asks) =>
        set((state) => {
          if (state.selectedMarketId === marketId) {
            state.orderbook = {
              market_id: marketId,
              bids,
              asks,
              timestamp: Date.now(),
            };
          }
        }),

      addTrade: (trade) =>
        set((state) => {
          if (state.selectedMarketId === trade.market_id) {
            state.recentTrades.unshift(trade);
            if (state.recentTrades.length > 100) {
              state.recentTrades = state.recentTrades.slice(0, 100);
            }
          }
        }),

      // ========================================================================
      // User Data Actions
      // ========================================================================

      setUser: (address) =>
        set((state) => {
          state.userAddress = address;
          state.isAuthenticated = true;
        }),

      clearUser: () =>
        set((state) => {
          state.userAddress = null;
          state.isAuthenticated = false;
          state.userBalances = {};
          state.userOrders = {};
          state.userTrades = [];
        }),

      setBalances: (balances) =>
        set((state) => {
          // Convert array to Record keyed by token_ticker
          state.userBalances = balances.reduce(
            (acc, balance) => {
              acc[balance.token_ticker] = balance;
              return acc;
            },
            {} as Record<string, Balance>
          );
        }),

      updateBalance: (tokenTicker, available, locked) =>
        set((state) => {
          const existing = state.userBalances[tokenTicker];
          const totalAmount = (BigInt(available) + BigInt(locked)).toString();
          const token = state.tokens[tokenTicker];
          if (!token) return;

          const divisor = Math.pow(10, token.decimals);
          const amountValue = Number(BigInt(totalAmount)) / divisor;
          const lockedValue = Number(BigInt(locked)) / divisor;

          // O(1) insert or update - handles both new and existing balances
          state.userBalances[tokenTicker] = {
            token_ticker: tokenTicker,
            user_address: existing?.user_address || state.userAddress || "",
            amount: totalAmount,
            open_interest: locked,
            updated_at: new Date(),
            amountDisplay: amountValue.toFixed(token.decimals),
            lockedDisplay: lockedValue.toFixed(token.decimals),
            amountValue,
            lockedValue,
          };
        }),

      setOrders: (orders) =>
        set((state) => {
          // Convert array to Record keyed by order id
          state.userOrders = orders.reduce(
            (acc, order) => {
              acc[order.id] = order;
              return acc;
            },
            {} as Record<string, Order>
          );
        }),

      updateOrder: (orderId, status, filledSize) =>
        set((state) => {
          const existing = state.userOrders[orderId];
          if (!existing) {
            // Order not in store yet - it will be fetched on next refetch
            console.warn(`Order ${orderId} not found in store for update`);
            return;
          }

          const market = state.markets[existing.market_id];
          if (!market) return;

          const baseToken = state.tokens[market.base_ticker];
          if (!baseToken) return;

          const divisor = Math.pow(10, baseToken.decimals);
          const filledValue = Number(BigInt(filledSize)) / divisor;

          // O(1) update - directly update the order in the Record
          state.userOrders[orderId] = {
            ...existing,
            status,
            filled_size: filledSize,
            filledDisplay: filledValue.toFixed(baseToken.decimals),
            filledValue,
          };
        }),

      setUserTrades: (trades) =>
        set((state) => {
          state.userTrades = trades;
        }),

      addUserTrade: (trade) =>
        set((state) => {
          state.userTrades.unshift(trade);
          if (state.userTrades.length > 100) {
            state.userTrades = state.userTrades.slice(0, 100);
          }
        }),

      // ========================================================================
      // Utilities
      // ========================================================================

      reset: () => set(initialState),
    })),
    { name: "ExchangeStore" }
  )
);

// ============================================================================
// Selectors (for optimized re-renders)
// ============================================================================

// Stable empty arrays to prevent unnecessary re-renders
const EMPTY_ARRAY: OrderbookLevel[] = [];

export const selectSelectedMarket = (state: ExchangeState) =>
  state.selectedMarketId ? state.markets[state.selectedMarketId] : undefined;

export const selectOrderbookBids = (state: ExchangeState) => state.orderbook?.bids ?? EMPTY_ARRAY;

export const selectOrderbookAsks = (state: ExchangeState) => state.orderbook?.asks ?? EMPTY_ARRAY;

export const selectRecentTrades = (state: ExchangeState) => state.recentTrades;
