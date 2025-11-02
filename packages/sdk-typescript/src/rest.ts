/**
 * REST API client for the exchange
 */

import type { components } from './types/generated';
import { ApiError } from './errors';
import { toDisplayValue, formatPrice, formatSize } from './format';

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
type CandlesRequest = components['schemas']['CandlesRequest'];

// Domain types (from API)
export type Market = components['schemas']['ApiMarket'];
export type Token = components['schemas']['Token'];
export type Order = components['schemas']['ApiOrder'];
export type Trade = components['schemas']['ApiTrade'];
export type Balance = components['schemas']['ApiBalance'];
export type Side = components['schemas']['Side'];
export type OrderType = components['schemas']['OrderType'];
export type OrderStatus = components['schemas']['OrderStatus'];
export type Candle = components['schemas']['ApiCandle'];
export type CandlesResponse = components['schemas']['CandlesResponse'];

// Enhanced types (with display values pre-computed)
export type EnhancedTrade = Omit<Trade, 'timestamp'> & {
  timestamp: Date;          // Converted from string
  priceDisplay: string;     // Formatted price
  sizeDisplay: string;      // Formatted size
  priceValue: number;       // Numeric price
  sizeValue: number;        // Numeric size
};

export type EnhancedOrder = Omit<Order, 'created_at' | 'updated_at'> & {
  created_at: Date;         // Converted from string
  updated_at: Date;         // Converted from string
  priceDisplay: string;     // Formatted price
  sizeDisplay: string;      // Formatted size
  filledDisplay: string;    // Formatted filled_size
  priceValue: number;       // Numeric price
  sizeValue: number;        // Numeric size
  filledValue: number;      // Numeric filled_size
};

export type EnhancedBalance = Omit<Balance, 'updated_at'> & {
  updated_at: Date;         // Converted from string
  amountDisplay: string;    // Formatted amount
  lockedDisplay: string;    // Formatted open_interest
  amountValue: number;      // Numeric amount
  lockedValue: number;      // Numeric open_interest
};

export interface EnhancedOrderbookLevel {
  price: string;            // atoms
  size: string;             // atoms
  priceDisplay: string;     // Formatted
  sizeDisplay: string;      // Formatted
  priceValue: number;       // Numeric
  sizeValue: number;        // Numeric
}

export interface RestClientConfig {
  baseUrl: string;
  timeout?: number;
}

export class RestClient {
  private baseUrl: string;
  private timeout: number;

  // Internal caches for data enhancement
  private tokensCache: Map<string, Token> = new Map();
  private marketsCache: Map<string, Market> = new Map();

  constructor(config: RestClientConfig) {
    if (!config.baseUrl) {
      throw new Error('RestClient: baseUrl is required');
    }
    this.baseUrl = config.baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.timeout = config.timeout ?? 30000;
  }

  // Public access to cached data
  get tokens(): Token[] {
    return Array.from(this.tokensCache.values());
  }

  get markets(): Market[] {
    return Array.from(this.marketsCache.values());
  }

  getTokenByTicker(ticker: string): Token | undefined {
    return this.tokensCache.get(ticker);
  }

  getMarketById(marketId: string): Market | undefined {
    return this.marketsCache.get(marketId);
  }

  // ===== Enhancement Helpers =====

  /**
   * Enhance a trade with display values
   * @public - Used by WebSocket handlers
   */
  public enhanceTrade(trade: Trade): EnhancedTrade {
    const market = this.marketsCache.get(trade.market_id);
    if (!market) {
      throw new Error(`Market ${trade.market_id} not found in cache. Call getMarkets() first.`);
    }

    const baseToken = this.tokensCache.get(market.base_ticker);
    const quoteToken = this.tokensCache.get(market.quote_ticker);

    if (!baseToken || !quoteToken) {
      throw new Error(`Tokens for market ${trade.market_id} not found in cache. Call getTokens() first.`);
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
   * Enhance an order with display values
   * @public - Used by WebSocket handlers
   */
  public enhanceOrder(order: Order, marketId: string): EnhancedOrder {
    const market = this.marketsCache.get(marketId);
    if (!market) {
      throw new Error(`Market ${marketId} not found in cache. Call getMarkets() first.`);
    }

    const baseToken = this.tokensCache.get(market.base_ticker);
    const quoteToken = this.tokensCache.get(market.quote_ticker);

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
   * @public - Used by WebSocket handlers
   */
  public enhanceBalance(balance: Balance): EnhancedBalance {
    const token = this.tokensCache.get(balance.token_ticker);
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

    // Cache markets for data enhancement
    response.markets.forEach(market => {
      this.marketsCache.set(market.id, market);
    });

    return response.markets;
  }

  async getTokens(): Promise<Token[]> {
    const request: InfoRequest = { type: 'all_tokens' };
    const response = await this.post<InfoResponse>('/api/info', request);
    if (response.type !== 'all_tokens') {
      throw new ApiError('Invalid response type', 500);
    }

    // Cache tokens for data enhancement
    response.tokens.forEach(token => {
      this.tokensCache.set(token.ticker, token);
    });

    return response.tokens;
  }

  // ===== User Endpoints =====

  async getOrders(params: {
    userAddress: string;
    marketId?: string;
    status?: OrderStatus;
    limit?: number;
  }): Promise<EnhancedOrder[]> {
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

    // Enhance orders with display values
    if (!params.marketId) {
      throw new Error('marketId is required for data enhancement');
    }
    return response.orders.map(order => this.enhanceOrder(order, params.marketId!));
  }

  async getBalances(userAddress: string): Promise<EnhancedBalance[]> {
    const request: UserRequest = {
      type: 'balances',
      user_address: userAddress,
    };
    const response = await this.post<UserResponse>('/api/user', request);
    if (response.type !== 'balances') {
      throw new ApiError('Invalid response type', 500);
    }

    // Enhance balances with display values
    return response.balances.map(balance => this.enhanceBalance(balance));
  }

  async getTrades(params: {
    userAddress: string;
    marketId?: string;
    limit?: number;
  }): Promise<EnhancedTrade[]> {
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

    // Enhance trades with display values
    return response.trades.map(trade => this.enhanceTrade(trade));
  }

  // ===== Trade Endpoints =====

  /**
   * Round a size to the nearest multiple of lot_size (rounds down)
   */
  static roundSizeToLot(size: bigint, lotSize: bigint): bigint {
    if (lotSize === 0n) {
      return size;
    }
    return (size / lotSize) * lotSize;
  }

  /**
   * Round a size string to the nearest multiple of lot_size (rounds down)
   */
  static roundSizeToLotStr(size: string, lotSize: string): string {
    const sizeVal = BigInt(size);
    const lotSizeVal = BigInt(lotSize);
    const rounded = RestClient.roundSizeToLot(sizeVal, lotSizeVal);
    return rounded.toString();
  }

  async placeOrder(params: {
    userAddress: string;
    marketId: string;
    side: Side;
    orderType: OrderType;
    price: string;
    size: string;
    signature: string;
  }): Promise<{ order: EnhancedOrder; trades: EnhancedTrade[] }> {
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

    // Enhance the order and trades
    return {
      order: this.enhanceOrder(response.order, params.marketId),
      trades: response.trades.map(trade => this.enhanceTrade(trade)),
    };
  }

  /**
   * Place an order with automatic size rounding to lot_size
   */
  async placeOrderWithRounding(params: {
    userAddress: string;
    marketId: string;
    side: Side;
    orderType: OrderType;
    price: string;
    size: string;
    signature: string;
  }): Promise<{ order: EnhancedOrder; trades: EnhancedTrade[] }> {
    // Get market details to find lot_size
    const market = await this.getMarket(params.marketId);

    // Parse size
    const sizeVal = BigInt(params.size);
    const lotSizeVal = BigInt(market.lot_size);

    // Round size to lot_size
    const roundedSize = RestClient.roundSizeToLot(sizeVal, lotSizeVal);

    // Check if rounded size is 0
    if (roundedSize === 0n) {
      throw new ApiError(
        `Size ${params.size} is too small for lot_size ${market.lot_size} (rounded to 0)`,
        400
      );
    }

    // Place order with rounded size
    return this.placeOrder({
      ...params,
      size: roundedSize.toString(),
    });
  }

  /**
   * Place an order with human-readable decimal values (e.g., "0.5" BTC, "110000" USDC)
   * Automatically converts to atoms using token decimals from market config
   */
  async placeOrderDecimal(params: {
    userAddress: string;
    marketId: string;
    side: Side;
    orderType: OrderType;
    priceDecimal: string; // Human-readable price (e.g., "110000.50")
    sizeDecimal: string;  // Human-readable size (e.g., "0.5")
    signature: string;
  }): Promise<{ order: EnhancedOrder; trades: EnhancedTrade[] }> {
    // Get market and token details
    const market = await this.getMarket(params.marketId);
    const baseToken = await this.getToken(market.base_ticker);
    const quoteToken = await this.getToken(market.quote_ticker);

    // Convert price from decimal to atoms using quote token decimals
    const priceDecimal = parseFloat(params.priceDecimal);
    const priceMultiplier = Math.pow(10, quoteToken.decimals);
    const priceAtoms = BigInt(Math.floor(priceDecimal * priceMultiplier));

    // Convert size from decimal to atoms using base token decimals
    const sizeDecimal = parseFloat(params.sizeDecimal);
    const sizeMultiplier = Math.pow(10, baseToken.decimals);
    const sizeAtoms = BigInt(Math.floor(sizeDecimal * sizeMultiplier));

    // Round size to lot_size
    const lotSizeVal = BigInt(market.lot_size);
    const roundedSize = RestClient.roundSizeToLot(sizeAtoms, lotSizeVal);

    // Check minimum size
    const minSizeVal = BigInt(market.min_size);
    if (roundedSize < minSizeVal) {
      throw new ApiError(
        `Size ${params.sizeDecimal} (${roundedSize} atoms) is below minimum ${market.min_size} atoms`,
        400
      );
    }

    // Place order with converted values
    return this.placeOrder({
      userAddress: params.userAddress,
      marketId: params.marketId,
      side: params.side,
      orderType: params.orderType,
      price: priceAtoms.toString(),
      size: roundedSize.toString(),
      signature: params.signature,
    });
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

  async cancelAllOrders(params: {
    userAddress: string;
    marketId?: string;
    signature: string;
  }): Promise<{ cancelledOrderIds: string[]; count: number }> {
    const request: TradeRequest = {
      type: 'cancel_all_orders',
      user_address: params.userAddress,
      market_id: params.marketId,
      signature: params.signature,
    };
    const response = await this.post<TradeResponse>('/api/trade', request);
    if (response.type !== 'cancel_all_orders') {
      throw new ApiError('Invalid response type', 500);
    }
    return { cancelledOrderIds: response.cancelled_order_ids, count: response.count };
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
    countBack?: number;
  }): Promise<Candle[]> {
    const request: CandlesRequest = {
      market_id: params.marketId,
      interval: params.interval,
      from: params.from,
      to: params.to,
      count_back: params.countBack,
    };
    const response = await this.post<CandlesResponse>('/api/candles', request);
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
