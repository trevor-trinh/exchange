/**
 * REST API client for the exchange
 */

import type { Market, Token, Balance, Order, Trade } from './types/exchange';

class APIError extends Error {
  constructor(
    message: string,
    public status: number,
    public code?: string
  ) {
    super(message);
    this.name = 'APIError';
  }
}

export class ExchangeAPI {
  private baseUrl: string;

  constructor(baseUrl?: string) {
    this.baseUrl = baseUrl || process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8888';
  }

  /**
   * Make a POST request to the API
   */
  private async post<T>(endpoint: string, body: any): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({
          error: response.statusText,
        }));
        throw new APIError(
          error.error || 'Request failed',
          response.status,
          error.code
        );
      }

      return response.json();
    } catch (error) {
      if (error instanceof APIError) {
        throw error;
      }
      throw new APIError(
        error instanceof Error ? error.message : 'Network error',
        0
      );
    }
  }

  /**
   * Make a GET request to the API
   */
  private async get<T>(endpoint: string): Promise<T> {
    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`);

      if (!response.ok) {
        const error = await response.json().catch(() => ({
          error: response.statusText,
        }));
        throw new APIError(
          error.error || 'Request failed',
          response.status,
          error.code
        );
      }

      return response.json();
    } catch (error) {
      if (error instanceof APIError) {
        throw error;
      }
      throw new APIError(
        error instanceof Error ? error.message : 'Network error',
        0
      );
    }
  }

  // ============================================================================
  // Health
  // ============================================================================

  async health(): Promise<{ status: string }> {
    return this.get('/api/health');
  }

  // ============================================================================
  // Info Endpoints
  // ============================================================================

  async getMarkets(): Promise<Market[]> {
    const response = await this.post<{ type: 'all_markets'; markets: Market[] }>(
      '/api/info',
      { type: 'all_markets' }
    );
    return response.markets;
  }

  async getMarket(marketId: string): Promise<Market> {
    const response = await this.post<{ type: 'market_details'; market: Market }>(
      '/api/info',
      { type: 'market_details', market_id: marketId }
    );
    return response.market;
  }

  async getTokens(): Promise<Token[]> {
    const response = await this.post<{ type: 'all_tokens'; tokens: Token[] }>(
      '/api/info',
      { type: 'all_tokens' }
    );
    return response.tokens;
  }

  async getToken(ticker: string): Promise<Token> {
    const response = await this.post<{ type: 'token_details'; token: Token }>(
      '/api/info',
      { type: 'token_details', ticker }
    );
    return response.token;
  }

  // ============================================================================
  // User Endpoints
  // ============================================================================

  async getBalances(userAddress: string): Promise<Balance[]> {
    const response = await this.post<{ type: 'balances'; balances: Balance[] }>(
      '/api/user',
      { type: 'balances', user_address: userAddress }
    );
    return response.balances;
  }

  async getOrders(userAddress: string, marketId?: string): Promise<Order[]> {
    const response = await this.post<{ type: 'orders'; orders: Order[] }>(
      '/api/user',
      {
        type: 'orders',
        user_address: userAddress,
        market_id: marketId,
      }
    );
    return response.orders;
  }

  async getTrades(userAddress: string, marketId?: string): Promise<Trade[]> {
    const response = await this.post<{ type: 'trades'; trades: Trade[] }>(
      '/api/user',
      {
        type: 'trades',
        user_address: userAddress,
        market_id: marketId,
      }
    );
    return response.trades;
  }

  // ============================================================================
  // Trading Endpoints
  // ============================================================================

  async placeOrder(
    userAddress: string,
    marketId: string,
    side: 'Buy' | 'Sell',
    orderType: 'Limit' | 'Market',
    price: string,
    size: string,
    signature: string
  ): Promise<{ order: Order; trades: Trade[] }> {
    const response = await this.post<{
      type: 'place_order';
      order: Order;
      trades: Trade[];
    }>('/api/trade', {
      type: 'place_order',
      user_address: userAddress,
      market_id: marketId,
      side,
      order_type: orderType,
      price,
      size,
      signature,
    });

    return { order: response.order, trades: response.trades };
  }

  async cancelOrder(
    userAddress: string,
    orderId: string,
    signature: string
  ): Promise<{ order_id: string }> {
    const response = await this.post<{
      type: 'cancel_order';
      order_id: string;
    }>('/api/trade', {
      type: 'cancel_order',
      user_address: userAddress,
      order_id: orderId,
      signature,
    });

    return { order_id: response.order_id };
  }

  // ============================================================================
  // Admin Endpoints (for setup/testing)
  // ============================================================================

  async adminCreateToken(
    ticker: string,
    decimals: number,
    name: string
  ): Promise<Token> {
    const response = await this.post<{ type: 'create_token'; token: Token }>(
      '/api/admin',
      {
        type: 'create_token',
        ticker,
        decimals,
        name,
      }
    );
    return response.token;
  }

  async adminCreateMarket(
    baseTicker: string,
    quoteTicker: string,
    tickSize: number,
    lotSize: number,
    minSize: number,
    makerFeeBps: number,
    takerFeeBps: number
  ): Promise<Market> {
    const response = await this.post<{ type: 'create_market'; market: Market }>(
      '/api/admin',
      {
        type: 'create_market',
        base_ticker: baseTicker,
        quote_ticker: quoteTicker,
        tick_size: tickSize,
        lot_size: lotSize,
        min_size: minSize,
        maker_fee_bps: makerFeeBps,
        taker_fee_bps: takerFeeBps,
      }
    );
    return response.market;
  }

  async adminFaucet(
    userAddress: string,
    tokenTicker: string,
    amount: string
  ): Promise<string> {
    const response = await this.post<{
      type: 'faucet';
      new_balance: string;
    }>('/api/admin', {
      type: 'faucet',
      user_address: userAddress,
      token_ticker: tokenTicker,
      amount,
      signature: 'admin',
    });
    return response.new_balance;
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

// Export error class
export { APIError };
