/**
 * REST API client for the exchange
 * Now using the @exchange/sdk package
 */

import { RestClient, ApiError } from "@exchange/sdk";
import type { Market, Token, Balance, Order, Trade } from "./types/exchange";

export class ExchangeAPI {
  private client: RestClient;

  constructor(baseUrl?: string) {
    const url = baseUrl || process.env.NEXT_PUBLIC_API_URL || "http://localhost:8888";
    this.client = new RestClient({ baseUrl: url });
  }

  // ============================================================================
  // Info Endpoints
  // ============================================================================

  async getMarkets(): Promise<Market[]> {
    return this.client.getMarkets();
  }

  async getMarket(marketId: string): Promise<Market> {
    return this.client.getMarket(marketId);
  }

  async getTokens(): Promise<Token[]> {
    return this.client.getTokens();
  }

  async getToken(ticker: string): Promise<Token> {
    return this.client.getToken(ticker);
  }

  // ============================================================================
  // User Endpoints
  // ============================================================================

  async getBalances(userAddress: string): Promise<Balance[]> {
    return this.client.getBalances(userAddress);
  }

  async getOrders(userAddress: string, marketId?: string): Promise<Order[]> {
    return this.client.getOrders({ userAddress, marketId });
  }

  async getTrades(userAddress: string, marketId?: string): Promise<Trade[]> {
    return this.client.getTrades({ userAddress, marketId });
  }

  // ============================================================================
  // Trading Endpoints
  // ============================================================================

  async placeOrder(
    userAddress: string,
    marketId: string,
    side: "Buy" | "Sell",
    orderType: "Limit" | "Market",
    price: string,
    size: string,
    signature: string,
  ): Promise<{ order: Order; trades: Trade[] }> {
    return this.client.placeOrder({
      userAddress,
      marketId,
      side,
      orderType,
      price,
      size,
      signature,
    });
  }

  async cancelOrder(userAddress: string, orderId: string, signature: string): Promise<{ orderId: string }> {
    return this.client.cancelOrder({
      userAddress,
      orderId,
      signature,
    });
  }

  // ============================================================================
  // Admin Endpoints (for setup/testing)
  // ============================================================================

  async adminCreateToken(ticker: string, decimals: number, name: string): Promise<Token> {
    return this.client.adminCreateToken({ ticker, decimals, name });
  }

  async adminCreateMarket(
    baseTicker: string,
    quoteTicker: string,
    tickSize: number,
    lotSize: number,
    minSize: number,
    makerFeeBps: number,
    takerFeeBps: number,
  ): Promise<Market> {
    return this.client.adminCreateMarket({
      baseTicker,
      quoteTicker,
      tickSize: tickSize.toString(),
      lotSize: lotSize.toString(),
      minSize: minSize.toString(),
      makerFeeBps,
      takerFeeBps,
    });
  }

  async adminFaucet(userAddress: string, tokenTicker: string, amount: string): Promise<string> {
    const result = await this.client.adminFaucet({
      userAddress,
      tokenTicker,
      amount,
    });
    return result.newBalance;
  }
}

// Singleton instance
let apiInstance: ExchangeAPI | null = null;

export function getAPI(): ExchangeAPI {
  if (!apiInstance) {
    apiInstance = new ExchangeAPI();
  }
  return apiInstance;
}

// Export error class from SDK
export { ApiError };
