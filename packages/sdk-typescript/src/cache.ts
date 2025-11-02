/**
 * Cache service for markets and tokens
 */

import type { components } from "./types/generated";
import type { Logger } from "./logger";

export type Market = components["schemas"]["ApiMarket"];
export type Token = components["schemas"]["Token"];

/**
 * Cache service for storing and retrieving markets and tokens
 */
export class CacheService {
  private tokensCache = new Map<string, Token>();
  private marketsCache = new Map<string, Market>();
  private initialized = false;
  private logger: Logger;

  constructor(logger: Logger) {
    this.logger = logger;
  }

  // ===== Tokens =====

  setTokens(tokens: Token[]): void {
    this.tokensCache.clear();
    tokens.forEach((token) => {
      this.tokensCache.set(token.ticker, token);
    });
    this.logger.debug(`Cached ${tokens.length} tokens`);
  }

  getToken(ticker: string): Token | undefined {
    return this.tokensCache.get(ticker);
  }

  getAllTokens(): Token[] {
    return Array.from(this.tokensCache.values());
  }

  hasToken(ticker: string): boolean {
    return this.tokensCache.has(ticker);
  }

  // ===== Markets =====

  setMarkets(markets: Market[]): void {
    this.marketsCache.clear();
    markets.forEach((market) => {
      this.marketsCache.set(market.id, market);
    });
    this.logger.debug(`Cached ${markets.length} markets`);
  }

  getMarket(marketId: string): Market | undefined {
    return this.marketsCache.get(marketId);
  }

  getAllMarkets(): Market[] {
    return Array.from(this.marketsCache.values());
  }

  hasMarket(marketId: string): boolean {
    return this.marketsCache.has(marketId);
  }

  // ===== Cache State =====

  isReady(): boolean {
    return this.initialized && this.tokensCache.size > 0 && this.marketsCache.size > 0;
  }

  markInitialized(): void {
    this.initialized = true;
    this.logger.info(`Cache initialized: ${this.marketsCache.size} markets, ${this.tokensCache.size} tokens`);
  }

  clear(): void {
    this.tokensCache.clear();
    this.marketsCache.clear();
    this.initialized = false;
    this.logger.debug("Cache cleared");
  }

  getStats(): { markets: number; tokens: number; initialized: boolean } {
    return {
      markets: this.marketsCache.size,
      tokens: this.tokensCache.size,
      initialized: this.initialized,
    };
  }
}
