/**
 * Global state management with Zustand
 */

import { create } from "zustand";
import { devtools } from "zustand/middleware";
import { immer } from "zustand/middleware/immer";
import type { Market, Token, Orderbook, OrderbookLevel, Trade, PricePoint } from "./types/exchange";

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

  // Trades & Price history
  recentTrades: Trade[];
  priceHistory: PricePoint[];

  // Loading states
  isLoadingMarkets: boolean;
  isLoadingOrderbook: boolean;

  // Actions - Markets
  setMarkets: (markets: Market[]) => void;
  setTokens: (tokens: Token[]) => void;
  selectMarket: (marketId: string) => void;

  // Actions - Orderbook
  updateOrderbook: (marketId: string, bids: OrderbookLevel[], asks: OrderbookLevel[]) => void;
  setOrderbookLoading: (loading: boolean) => void;

  // Actions - Trades
  addTrade: (trade: Trade) => void;
  addTrades: (trades: Trade[]) => void;

  // Actions - Price history
  addPricePoint: (price: number) => void;

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
  priceHistory: [],
  isLoadingMarkets: false,
  isLoadingOrderbook: false,
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
          state.isLoadingMarkets = false;
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
          state.priceHistory = [];
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
            state.isLoadingOrderbook = false;
          }
        }),

      setOrderbookLoading: (loading) =>
        set((state) => {
          state.isLoadingOrderbook = loading;
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

            // Add to price history - convert using token decimals
            const market = state.markets.find((m) => m.id === trade.market_id);
            if (market) {
              const quoteToken = state.tokens.find((t) => t.ticker === market.quote_ticker);
              if (quoteToken) {
                const raw = parseFloat(trade.price);
                const price = raw / Math.pow(10, quoteToken.decimals);
                if (!isNaN(price)) {
                  state.priceHistory.push({
                    timestamp: Date.now(),
                    price,
                  });

                  // Keep only last 200 price points
                  if (state.priceHistory.length > 200) {
                    state.priceHistory = state.priceHistory.slice(-200);
                  }
                }
              }
            }
          }
        }),

      addTrades: (trades) =>
        set((state) => {
          trades.forEach((trade) => {
            // Only add if this is for the selected market
            if (state.selectedMarketId === trade.market_id) {
              state.recentTrades.unshift(trade);

              // Add to price history - convert using token decimals
              const market = state.markets.find((m) => m.id === trade.market_id);
              if (market) {
                const quoteToken = state.tokens.find((t) => t.ticker === market.quote_ticker);
                if (quoteToken) {
                  const raw = parseFloat(trade.price);
                  const price = raw / Math.pow(10, quoteToken.decimals);
                  if (!isNaN(price)) {
                    state.priceHistory.push({
                      timestamp: Date.now(),
                      price,
                    });
                  }
                }
              }
            }
          });

          // Keep only last 100 trades
          if (state.recentTrades.length > 100) {
            state.recentTrades = state.recentTrades.slice(0, 100);
          }

          // Keep only last 200 price points
          if (state.priceHistory.length > 200) {
            state.priceHistory = state.priceHistory.slice(-200);
          }
        }),

      // Price history
      addPricePoint: (price) =>
        set((state) => {
          state.priceHistory.push({
            timestamp: Date.now(),
            price,
          });

          // Keep only last 200 points
          if (state.priceHistory.length > 200) {
            state.priceHistory = state.priceHistory.slice(-200);
          }
        }),

      // Reset
      reset: () => set(initialState),
    })),
    { name: "ExchangeStore" },
  ),
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

export const selectPriceHistory = (state: ExchangeState) => state.priceHistory;

export const selectCurrentPrice = (state: ExchangeState) => {
  if (state.priceHistory.length > 0) {
    return state.priceHistory[state.priceHistory.length - 1].price;
  }
  return null;
};
