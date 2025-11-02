/**
 * Service for enhancing raw API data with display values
 */

import type { components } from "./types/generated";
import type { CacheService } from "./cache";
import { toDisplayValue, formatPrice, formatSize } from "./format";

// Domain types (from API)
export type Order = components["schemas"]["ApiOrder"];
export type Trade = components["schemas"]["ApiTrade"];
export type Balance = components["schemas"]["ApiBalance"];

// Enhanced types (with display values pre-computed)
export type EnhancedTrade = Omit<Trade, "timestamp"> & {
  timestamp: Date; // Converted from string
  priceDisplay: string; // Formatted price
  sizeDisplay: string; // Formatted size
  priceValue: number; // Numeric price
  sizeValue: number; // Numeric size
};

export type EnhancedOrder = Omit<Order, "created_at" | "updated_at"> & {
  created_at: Date; // Converted from string
  updated_at: Date; // Converted from string
  priceDisplay: string; // Formatted price
  sizeDisplay: string; // Formatted size
  filledDisplay: string; // Formatted filled_size
  priceValue: number; // Numeric price
  sizeValue: number; // Numeric size
  filledValue: number; // Numeric filled_size
};

export type EnhancedBalance = Omit<Balance, "updated_at"> & {
  updated_at: Date; // Converted from string
  amountDisplay: string; // Formatted amount
  lockedDisplay: string; // Formatted open_interest
  amountValue: number; // Numeric amount
  lockedValue: number; // Numeric open_interest
};

export interface EnhancedOrderbookLevel {
  price: string; // atoms
  size: string; // atoms
  priceDisplay: string; // Formatted
  sizeDisplay: string; // Formatted
  priceValue: number; // Numeric
  sizeValue: number; // Numeric
}

/**
 * WebSocket trade data (uses Unix timestamp in seconds instead of ISO string)
 */
export interface WsTradeData {
  id: string;
  market_id: string;
  buyer_address: string;
  seller_address: string;
  buyer_order_id: string;
  seller_order_id: string;
  price: string;
  size: string;
  side: "buy" | "sell";
  timestamp: number; // Unix timestamp in seconds
}

/**
 * Service for enhancing raw data with display values and conversions
 */
export class EnhancementService {
  constructor(private cache: CacheService) {}

  /**
   * Enhance a REST trade with display values
   */
  enhanceTrade(trade: Trade): EnhancedTrade {
    const market = this.cache.getMarket(trade.market_id);
    if (!market) {
      throw new Error(
        `Market ${trade.market_id} not found in cache. ` +
          `Available markets: ${
            this.cache
              .getAllMarkets()
              .map((m) => m.id)
              .join(", ") || "none"
          }. ` +
          `Call getMarkets() first to populate cache.`
      );
    }

    const baseToken = this.cache.getToken(market.base_ticker);
    const quoteToken = this.cache.getToken(market.quote_ticker);

    if (!baseToken || !quoteToken) {
      throw new Error(
        `Tokens for market ${trade.market_id} not found in cache. ` +
          `Need: ${market.base_ticker}, ${market.quote_ticker}. ` +
          `Available: ${
            this.cache
              .getAllTokens()
              .map((t) => t.ticker)
              .join(", ") || "none"
          }. ` +
          `Call getTokens() first to populate cache.`
      );
    }

    return {
      ...trade,
      timestamp: new Date(trade.timestamp),
      priceDisplay: formatPrice(trade.price, quoteToken.decimals),
      sizeDisplay: formatSize(trade.size, baseToken.decimals),
      priceValue: toDisplayValue(trade.price, quoteToken.decimals),
      sizeValue: toDisplayValue(trade.size, baseToken.decimals),
    };
  }

  /**
   * Enhance a WebSocket trade with display values
   * WebSocket trades use Unix timestamps (seconds) instead of ISO strings
   */
  enhanceWsTrade(trade: WsTradeData): EnhancedTrade {
    // Convert WebSocket trade to REST trade format
    const restTrade: Trade = {
      id: trade.id,
      market_id: trade.market_id,
      buyer_address: trade.buyer_address,
      seller_address: trade.seller_address,
      buyer_order_id: trade.buyer_order_id,
      seller_order_id: trade.seller_order_id,
      price: trade.price,
      size: trade.size,
      side: trade.side,
      timestamp: new Date(trade.timestamp * 1000).toISOString(),
    };

    return this.enhanceTrade(restTrade);
  }

  /**
   * Enhance an order with display values
   */
  enhanceOrder(order: Order, marketId: string): EnhancedOrder {
    const market = this.cache.getMarket(marketId);
    if (!market) {
      throw new Error(`Market ${marketId} not found in cache. Call getMarkets() first.`);
    }

    const baseToken = this.cache.getToken(market.base_ticker);
    const quoteToken = this.cache.getToken(market.quote_ticker);

    if (!baseToken || !quoteToken) {
      throw new Error(`Tokens for market ${marketId} not found in cache. Call getTokens() first.`);
    }

    return {
      ...order,
      created_at: new Date(order.created_at),
      updated_at: new Date(order.updated_at),
      priceDisplay: formatPrice(order.price, quoteToken.decimals),
      sizeDisplay: formatSize(order.size, baseToken.decimals),
      filledDisplay: formatSize(order.filled_size, baseToken.decimals),
      priceValue: toDisplayValue(order.price, quoteToken.decimals),
      sizeValue: toDisplayValue(order.size, baseToken.decimals),
      filledValue: toDisplayValue(order.filled_size, baseToken.decimals),
    };
  }

  /**
   * Enhance a balance with display values
   */
  enhanceBalance(balance: Balance): EnhancedBalance {
    const token = this.cache.getToken(balance.token_ticker);
    if (!token) {
      throw new Error(`Token ${balance.token_ticker} not found in cache. Call getTokens() first.`);
    }

    return {
      ...balance,
      updated_at: new Date(balance.updated_at),
      amountDisplay: formatSize(balance.amount, token.decimals),
      lockedDisplay: formatSize(balance.open_interest, token.decimals),
      amountValue: toDisplayValue(balance.amount, token.decimals),
      lockedValue: toDisplayValue(balance.open_interest, token.decimals),
    };
  }

  /**
   * Enhance an orderbook level with display values
   */
  enhanceOrderbookLevel(level: { price: string; size: string }, marketId: string): EnhancedOrderbookLevel {
    const market = this.cache.getMarket(marketId);
    if (!market) {
      throw new Error(`Market ${marketId} not found in cache. Call getMarkets() first.`);
    }

    const baseToken = this.cache.getToken(market.base_ticker);
    const quoteToken = this.cache.getToken(market.quote_ticker);

    if (!baseToken || !quoteToken) {
      throw new Error(`Tokens for market ${marketId} not found in cache. Call getTokens() first.`);
    }

    return {
      ...level,
      priceDisplay: formatPrice(level.price, quoteToken.decimals),
      sizeDisplay: formatSize(level.size, baseToken.decimals),
      priceValue: toDisplayValue(level.price, quoteToken.decimals),
      sizeValue: toDisplayValue(level.size, baseToken.decimals),
    };
  }
}
