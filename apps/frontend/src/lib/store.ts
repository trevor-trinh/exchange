/**
 * Global state management with Zustand
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";
import type { Market, Token, Orderbook, Trade, OrderbookLevel, Balance } from "./types/exchange";

// ============================================================================
// State Interface
// ============================================================================

interface ExchangeState {
  // Market data
  markets: Market[];
  tokens: Token[];
  selectedMarketId: string | null;

  // Orderbook
  orderbook: Orderbook | null;

  // Trades
  recentTrades: Trade[];

  // User authentication
  userAddress: string | null;
  isAuthenticated: boolean;

  // User balances
  balances: Balance[];

  // UI state
  selectedPrice: number | null;

  // Actions - Markets
  setMarkets: (markets: Market[]) => void;
  setTokens: (tokens: Token[]) => void;
  selectMarket: (marketId: string) => void;

  // Actions - Orderbook
  updateOrderbook: (marketId: string, bids: OrderbookLevel[], asks: OrderbookLevel[]) => void;

  // Actions - Trades
  addTrade: (trade: Trade) => void;
  addTrades: (trades: Trade[]) => void;

  // Actions - User authentication
  setUser: (address: string) => void;
  clearUser: () => void;

  // Actions - Balances
  setBalances: (balances: Balance[]) => void;
  updateBalance: (tokenTicker: string, available: string, locked: string) => void;

  // Actions - UI state
  setSelectedPrice: (price: number | null) => void;

  // Utilities
  reset: () => void;
}

// ============================================================================
// Initial State
// ============================================================================

const initialState = {
  markets: [],
  tokens: [],
  selectedMarketId: null,
  orderbook: null,
  recentTrades: [],
  userAddress: null,
  isAuthenticated: false,
  balances: [],
  selectedPrice: null,
};

// ============================================================================
// Store
// ============================================================================

export const useExchangeStore = create<ExchangeState>()(
  devtools(
    immer((set) => ({
      ...initialState,

      // Market actions
      setMarkets: (markets) =>
        set((state) => {
          state.markets = markets;
        }),

      setTokens: (tokens) =>
        set((state) => {
          state.tokens = tokens;
        }),

      selectMarket: (marketId) =>
        set((state) => {
          state.selectedMarketId = marketId;
          // Clear orderbook and trades when switching markets
          state.orderbook = null;
          state.recentTrades = [];
        }),

      // Orderbook actions
      updateOrderbook: (marketId, bids, asks) =>
        set((state) => {
          // Only update if this is for the selected market
          if (state.selectedMarketId === marketId) {
            state.orderbook = {
              market_id: marketId,
              bids,
              asks,
              timestamp: Date.now(),
            };
          }
        }),

      // Trade actions
      addTrade: (trade) =>
        set((state) => {
          // Only add if this is for the selected market
          if (state.selectedMarketId === trade.market_id) {
            // Add to beginning
            state.recentTrades.unshift(trade);

            // Keep only last 100 trades
            if (state.recentTrades.length > 100) {
              state.recentTrades = state.recentTrades.slice(0, 100);
            }
          }
        }),

      addTrades: (trades) =>
        set((state) => {
          trades.forEach((trade) => {
            // Only add if this is for the selected market
            if (state.selectedMarketId === trade.market_id) {
              state.recentTrades.unshift(trade);
            }
          });

          // Keep only last 100 trades
          if (state.recentTrades.length > 100) {
            state.recentTrades = state.recentTrades.slice(0, 100);
          }
        }),

      // User authentication
      setUser: (address) =>
        set((state) => {
          state.userAddress = address;
          state.isAuthenticated = true;
        }),

      clearUser: () =>
        set((state) => {
          state.userAddress = null;
          state.isAuthenticated = false;
          state.balances = []; // Clear balances when user logs out
        }),

      // Balance actions
      setBalances: (balances) =>
        set((state) => {
          state.balances = balances;
        }),

      updateBalance: (tokenTicker, available, locked) =>
        set((state) => {
          const existingIndex = state.balances.findIndex((b) => b.token_ticker === tokenTicker);

          if (existingIndex >= 0 && state.balances[existingIndex]) {
            const existing = state.balances[existingIndex];
            const totalAmount = (BigInt(available) + BigInt(locked)).toString();

            // Get token for display conversion
            const token = state.tokens.find((t) => t.ticker === tokenTicker);
            if (!token) return;

            const divisor = Math.pow(10, token.decimals);
            const amountValue = Number(BigInt(totalAmount)) / divisor;
            const lockedValue = Number(BigInt(locked)) / divisor;

            // Immutable update: create a new array with the updated balance
            state.balances = state.balances.map((balance, index) =>
              index === existingIndex
                ? {
                    token_ticker: existing.token_ticker,
                    user_address: existing.user_address,
                    amount: totalAmount,
                    open_interest: locked,
                    updated_at: new Date(),
                    amountDisplay: amountValue.toFixed(token.decimals),
                    lockedDisplay: lockedValue.toFixed(token.decimals),
                    amountValue,
                    lockedValue,
                  }
                : balance
            );
          }
        }),

      // UI state
      setSelectedPrice: (price) =>
        set((state) => {
          state.selectedPrice = price;
        }),

      // Reset
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
  state.markets.find((m) => m.id === state.selectedMarketId);

export const selectOrderbookBids = (state: ExchangeState) => state.orderbook?.bids ?? EMPTY_ARRAY;

export const selectOrderbookAsks = (state: ExchangeState) => state.orderbook?.asks ?? EMPTY_ARRAY;

export const selectRecentTrades = (state: ExchangeState) => state.recentTrades;
