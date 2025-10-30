/**
 * REST API client for the exchange
 */

import type { components } from './types/generated';
import { ApiError } from './errors';

// Extract types from OpenAPI components
type InfoRequest = components['schemas']['InfoRequest'];
type InfoResponse = components['schemas']['InfoResponse'];
type UserRequest = components['schemas']['UserRequest'];
type UserResponse = components['schemas']['UserResponse'];
type TradeRequest = components['schemas']['TradeRequest'];
type TradeResponse = components['schemas']['TradeResponse'];
type DripRequest = components['schemas']['DripRequest'];
type DripResponse = components['schemas']['DripResponse'];
type AdminRequest = components['schemas']['AdminRequest'];
type AdminResponse = components['schemas']['AdminResponse'];

// Domain types
export type Market = components['schemas']['ApiMarket'];
export type Token = components['schemas']['Token'];
export type Order = components['schemas']['ApiOrder'];
export type Trade = components['schemas']['ApiTrade'];
export type Balance = components['schemas']['ApiBalance'];
export type Side = components['schemas']['Side'];
export type OrderType = components['schemas']['OrderType'];
export type OrderStatus = components['schemas']['OrderStatus'];
export type Candle = components['schemas']['Candle'];
export type CandlesResponse = components['schemas']['CandlesResponse'];

export interface RestClientConfig {
  baseUrl: string;
  timeout?: number;
}

export class RestClient {
  private baseUrl: string;
  private timeout: number;

  constructor(config: RestClientConfig) {
    if (!config.baseUrl) {
      throw new Error('RestClient: baseUrl is required');
    }
    this.baseUrl = config.baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.timeout = config.timeout ?? 30000;
  }

  // ===== Info Endpoints =====

  async getToken(ticker: string): Promise<Token> {
    const request: InfoRequest = { type: 'token_details', ticker };
    const response = await this.post<InfoResponse>('/api/info', request);
    if (response.type !== 'token_details') {
      throw new ApiError('Invalid response type', 500);
    }
    return response.token;
  }

  async getMarket(marketId: string): Promise<Market> {
    const request: InfoRequest = { type: 'market_details', market_id: marketId };
    const response = await this.post<InfoResponse>('/api/info', request);
    if (response.type !== 'market_details') {
      throw new ApiError('Invalid response type', 500);
    }
    return response.market;
  }

  async getMarkets(): Promise<Market[]> {
    const request: InfoRequest = { type: 'all_markets' };
    const response = await this.post<InfoResponse>('/api/info', request);
    if (response.type !== 'all_markets') {
      throw new ApiError('Invalid response type', 500);
    }
    return response.markets;
  }

  async getTokens(): Promise<Token[]> {
    const request: InfoRequest = { type: 'all_tokens' };
    const response = await this.post<InfoResponse>('/api/info', request);
    if (response.type !== 'all_tokens') {
      throw new ApiError('Invalid response type', 500);
    }
    return response.tokens;
  }

  // ===== User Endpoints =====

  async getOrders(params: {
    userAddress: string;
    marketId?: string;
    status?: OrderStatus;
    limit?: number;
  }): Promise<Order[]> {
    const request: UserRequest = {
      type: 'orders',
      user_address: params.userAddress,
      market_id: params.marketId,
      status: params.status,
      limit: params.limit,
    };
    const response = await this.post<UserResponse>('/api/user', request);
    if (response.type !== 'orders') {
      throw new ApiError('Invalid response type', 500);
    }
    return response.orders;
  }

  async getBalances(userAddress: string): Promise<Balance[]> {
    const request: UserRequest = {
      type: 'balances',
      user_address: userAddress,
    };
    const response = await this.post<UserResponse>('/api/user', request);
    if (response.type !== 'balances') {
      throw new ApiError('Invalid response type', 500);
    }
    return response.balances;
  }

  async getTrades(params: {
    userAddress: string;
    marketId?: string;
    limit?: number;
  }): Promise<Trade[]> {
    const request: UserRequest = {
      type: 'trades',
      user_address: params.userAddress,
      market_id: params.marketId,
      limit: params.limit,
    };
    const response = await this.post<UserResponse>('/api/user', request);
    if (response.type !== 'trades') {
      throw new ApiError('Invalid response type', 500);
    }
    return response.trades;
  }

  // ===== Trade Endpoints =====

  async placeOrder(params: {
    userAddress: string;
    marketId: string;
    side: Side;
    orderType: OrderType;
    price: string;
    size: string;
    signature: string;
  }): Promise<{ order: Order; trades: Trade[] }> {
    const request: TradeRequest = {
      type: 'place_order',
      user_address: params.userAddress,
      market_id: params.marketId,
      side: params.side,
      order_type: params.orderType,
      price: params.price,
      size: params.size,
      signature: params.signature,
    };
    const response = await this.post<TradeResponse>('/api/trade', request);
    if (response.type !== 'place_order') {
      throw new ApiError('Invalid response type', 500);
    }
    return { order: response.order, trades: response.trades };
  }

  async cancelOrder(params: {
    userAddress: string;
    orderId: string;
    signature: string;
  }): Promise<{ orderId: string }> {
    const request: TradeRequest = {
      type: 'cancel_order',
      user_address: params.userAddress,
      order_id: params.orderId,
      signature: params.signature,
    };
    const response = await this.post<TradeResponse>('/api/trade', request);
    if (response.type !== 'cancel_order') {
      throw new ApiError('Invalid response type', 500);
    }
    return { orderId: response.order_id };
  }

  // ===== Drip/Faucet =====

  async faucet(params: {
    userAddress: string;
    tokenTicker: string;
    amount: string;
    signature: string;
  }): Promise<{ userAddress: string; tokenTicker: string; amount: string; newBalance: string }> {
    const request: DripRequest = {
      type: 'faucet',
      user_address: params.userAddress,
      token_ticker: params.tokenTicker,
      amount: params.amount,
      signature: params.signature,
    };
    const response = await this.post<DripResponse>('/api/drip', request);
    return {
      userAddress: response.user_address,
      tokenTicker: response.token_ticker,
      amount: response.amount,
      newBalance: response.new_balance,
    };
  }

  // ===== Admin Endpoints =====

  async adminCreateToken(params: {
    ticker: string;
    decimals: number;
    name: string;
  }): Promise<Token> {
    const request: AdminRequest = {
      type: 'create_token',
      ticker: params.ticker,
      decimals: params.decimals as any, // OpenAPI types u8 as number
      name: params.name,
    };
    const response = await this.post<AdminResponse>('/api/admin', request);
    if (response.type !== 'create_token') {
      throw new ApiError('Invalid response type', 500);
    }
    return response.token;
  }

  async adminCreateMarket(params: {
    baseTicker: string;
    quoteTicker: string;
    tickSize: string;
    lotSize: string;
    minSize: string;
    makerFeeBps: number;
    takerFeeBps: number;
  }): Promise<Market> {
    const request: AdminRequest = {
      type: 'create_market',
      base_ticker: params.baseTicker,
      quote_ticker: params.quoteTicker,
      tick_size: params.tickSize as any, // Backend expects string (u128)
      lot_size: params.lotSize as any,
      min_size: params.minSize as any,
      maker_fee_bps: params.makerFeeBps as any, // OpenAPI types i32 as number
      taker_fee_bps: params.takerFeeBps as any,
    };
    const response = await this.post<AdminResponse>('/api/admin', request);
    if (response.type !== 'create_market') {
      throw new ApiError('Invalid response type', 500);
    }
    return response.market;
  }

  async adminFaucet(params: {
    userAddress: string;
    tokenTicker: string;
    amount: string;
  }): Promise<{ newBalance: string }> {
    const request: AdminRequest = {
      type: 'faucet',
      user_address: params.userAddress,
      token_ticker: params.tokenTicker,
      amount: params.amount,
      signature: 'admin',
    };
    const response = await this.post<AdminResponse>('/api/admin', request);
    if (response.type !== 'faucet') {
      throw new ApiError('Invalid response type', 500);
    }
    return { newBalance: response.new_balance };
  }

  // ===== Candles Endpoints =====

  async getCandles(params: {
    marketId: string;
    interval: string;
    from: number;
    to: number;
  }): Promise<Candle[]> {
    const queryParams = new URLSearchParams({
      market_id: params.marketId,
      interval: params.interval,
      from: params.from.toString(),
      to: params.to.toString(),
    });
    const response = await this.get<CandlesResponse>(`/api/candles?${queryParams}`);
    return response.candles;
  }

  // ===== HTTP Helpers =====

  private async get<T>(path: string): Promise<T> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: response.statusText }));
        throw new ApiError(
          error.error || 'Request failed',
          response.status,
          error.code
        );
      }

      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof ApiError) {
        throw error;
      }
      throw new ApiError('Network error', 0, undefined, error);
    }
  }

  private async post<T>(path: string, body: unknown): Promise<T> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(body),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: response.statusText }));
        throw new ApiError(
          error.error || 'Request failed',
          response.status,
          error.code
        );
      }

      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof ApiError) {
        throw error;
      }
      throw new ApiError('Network error', 0, undefined, error);
    }
  }
}
