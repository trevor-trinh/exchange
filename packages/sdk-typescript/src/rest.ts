/**
 * REST API client for the exchange
 */

import type { components } from "./types/generated";
import { ApiError } from "./errors";
import type { CacheService, Market, Token } from "./cache";
import type {
  EnhancementService,
  EnhancedTrade,
  EnhancedOrder,
  EnhancedBalance,
  EnhancedOrderbookLevel,
  Order,
  Trade,
  Balance,
} from "./enhancement";
import type { Logger } from "./logger";

// Extract types from OpenAPI components
type InfoRequest = components["schemas"]["InfoRequest"];
type InfoResponse = components["schemas"]["InfoResponse"];
type UserRequest = components["schemas"]["UserRequest"];
type UserResponse = components["schemas"]["UserResponse"];
type TradeRequest = components["schemas"]["TradeRequest"];
type TradeResponse = components["schemas"]["TradeResponse"];
type DripRequest = components["schemas"]["DripRequest"];
type DripResponse = components["schemas"]["DripResponse"];
type AdminRequest = components["schemas"]["AdminRequest"];
type AdminResponse = components["schemas"]["AdminResponse"];
type CandlesRequest = components["schemas"]["CandlesRequest"];

// Re-export domain types for convenience
export type { Market, Token };
export type { EnhancedTrade, EnhancedOrder, EnhancedBalance, EnhancedOrderbookLevel, Order, Trade, Balance };

export type Side = components["schemas"]["Side"];
export type OrderType = components["schemas"]["OrderType"];
export type OrderStatus = components["schemas"]["OrderStatus"];
export type Candle = components["schemas"]["ApiCandle"];
export type CandlesResponse = components["schemas"]["CandlesResponse"];

export interface RestClientConfig {
  baseUrl: string;
  timeout?: number;
  cache: CacheService;
  enhancer: EnhancementService;
  logger: Logger;
}

export class RestClient {
  private baseUrl: string;
  private timeout: number;
  private cache: CacheService;
  private enhancer: EnhancementService;
  private logger: Logger;

  constructor(config: RestClientConfig) {
    if (!config.baseUrl) {
      throw new Error("RestClient: baseUrl is required");
    }
    this.baseUrl = config.baseUrl.replace(/\/$/, ""); // Remove trailing slash
    this.timeout = config.timeout ?? 30000;
    this.cache = config.cache;
    this.enhancer = config.enhancer;
    this.logger = config.logger;
  }

  // ===== Info Endpoints =====

  async getToken(ticker: string): Promise<Token> {
    const request: InfoRequest = { type: "token_details", ticker };
    const response = await this.post<InfoResponse>("/api/info", request);
    if (response.type !== "token_details") {
      throw new ApiError("Invalid response type", 500);
    }
    return response.token;
  }

  async getMarket(marketId: string): Promise<Market> {
    const request: InfoRequest = {
      type: "market_details",
      market_id: marketId,
    };
    const response = await this.post<InfoResponse>("/api/info", request);
    if (response.type !== "market_details") {
      throw new ApiError("Invalid response type", 500);
    }
    return response.market;
  }

  async getMarkets(): Promise<Market[]> {
    this.logger.info("Fetching markets...");
    const request: InfoRequest = { type: "all_markets" };
    const response = await this.post<InfoResponse>("/api/info", request);
    if (response.type !== "all_markets") {
      throw new ApiError("Invalid response type", 500);
    }

    // Cache markets for data enhancement
    this.cache.setMarkets(response.markets);

    return response.markets;
  }

  async getTokens(): Promise<Token[]> {
    this.logger.info("Fetching tokens...");
    const request: InfoRequest = { type: "all_tokens" };
    const response = await this.post<InfoResponse>("/api/info", request);
    if (response.type !== "all_tokens") {
      throw new ApiError("Invalid response type", 500);
    }

    // Cache tokens for data enhancement
    this.cache.setTokens(response.tokens);

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
      type: "orders",
      user_address: params.userAddress,
      market_id: params.marketId,
      status: params.status,
      limit: params.limit,
    };
    const response = await this.post<UserResponse>("/api/user", request);
    if (response.type !== "orders") {
      throw new ApiError("Invalid response type", 500);
    }

    // Enhance orders with display values using each order's market_id
    return response.orders.map((order) => this.enhancer.enhanceOrder(order, order.market_id));
  }

  async getBalances(userAddress: string): Promise<EnhancedBalance[]> {
    const request: UserRequest = {
      type: "balances",
      user_address: userAddress,
    };
    const response = await this.post<UserResponse>("/api/user", request);
    if (response.type !== "balances") {
      throw new ApiError("Invalid response type", 500);
    }

    // Enhance balances with display values
    return response.balances.map((balance) => this.enhancer.enhanceBalance(balance));
  }

  async getTrades(params: { userAddress: string; marketId?: string; limit?: number }): Promise<EnhancedTrade[]> {
    const request: UserRequest = {
      type: "trades",
      user_address: params.userAddress,
      market_id: params.marketId,
      limit: params.limit,
    };
    const response = await this.post<UserResponse>("/api/user", request);
    if (response.type !== "trades") {
      throw new ApiError("Invalid response type", 500);
    }

    // Enhance trades with display values
    return response.trades.map((trade) => this.enhancer.enhanceTrade(trade));
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
      type: "place_order",
      user_address: params.userAddress,
      market_id: params.marketId,
      side: params.side,
      order_type: params.orderType,
      price: params.price,
      size: params.size,
      signature: params.signature,
    };
    const response = await this.post<TradeResponse>("/api/trade", request);
    if (response.type !== "place_order") {
      throw new ApiError("Invalid response type", 500);
    }

    // Enhance the order and trades
    return {
      order: this.enhancer.enhanceOrder(response.order, params.marketId),
      trades: response.trades.map((trade) => this.enhancer.enhanceTrade(trade)),
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
      throw new ApiError(`Size ${params.size} is too small for lot_size ${market.lot_size} (rounded to 0)`, 400);
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
    sizeDecimal: string; // Human-readable size (e.g., "0.5")
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

  async cancelOrder(params: { userAddress: string; orderId: string; signature: string }): Promise<{ orderId: string }> {
    const request: TradeRequest = {
      type: "cancel_order",
      user_address: params.userAddress,
      order_id: params.orderId,
      signature: params.signature,
    };
    const response = await this.post<TradeResponse>("/api/trade", request);
    if (response.type !== "cancel_order") {
      throw new ApiError("Invalid response type", 500);
    }
    return { orderId: response.order_id };
  }

  async cancelAllOrders(params: {
    userAddress: string;
    marketId?: string;
    signature: string;
  }): Promise<{ cancelledOrderIds: string[]; count: number }> {
    const request: TradeRequest = {
      type: "cancel_all_orders",
      user_address: params.userAddress,
      market_id: params.marketId,
      signature: params.signature,
    };
    const response = await this.post<TradeResponse>("/api/trade", request);
    if (response.type !== "cancel_all_orders") {
      throw new ApiError("Invalid response type", 500);
    }
    return {
      cancelledOrderIds: response.cancelled_order_ids,
      count: response.count,
    };
  }

  // ===== Drip/Faucet =====

  async faucet(params: { userAddress: string; tokenTicker: string; amount: string; signature: string }): Promise<{
    userAddress: string;
    tokenTicker: string;
    amount: string;
    newBalance: string;
  }> {
    const request: DripRequest = {
      type: "faucet",
      user_address: params.userAddress,
      token_ticker: params.tokenTicker,
      amount: params.amount,
      signature: params.signature,
    };
    const response = await this.post<DripResponse>("/api/drip", request);
    return {
      userAddress: response.user_address,
      tokenTicker: response.token_ticker,
      amount: response.amount,
      newBalance: response.new_balance,
    };
  }

  /**
   * Request tokens from faucet with human-readable decimal values
   * Automatically converts to atoms using token decimals
   */
  async faucetDecimal(params: {
    userAddress: string;
    tokenTicker: string;
    amountDecimal: string | number;
    signature: string;
  }): Promise<{
    userAddress: string;
    tokenTicker: string;
    amount: string;
    newBalance: string;
  }> {
    // Get token details to find decimals
    const token = await this.getToken(params.tokenTicker);

    // Convert amount from decimal to atoms
    const amountDecimal = typeof params.amountDecimal === "string" ? parseFloat(params.amountDecimal) : params.amountDecimal;
    const amountMultiplier = Math.pow(10, token.decimals);
    const amountAtoms = BigInt(Math.floor(amountDecimal * amountMultiplier));

    // Call the regular faucet with converted values
    return this.faucet({
      userAddress: params.userAddress,
      tokenTicker: params.tokenTicker,
      amount: amountAtoms.toString(),
      signature: params.signature,
    });
  }

  // ===== Admin Endpoints =====

  async adminCreateToken(params: { ticker: string; decimals: number; name: string }): Promise<Token> {
    const request: AdminRequest = {
      type: "create_token",
      ticker: params.ticker,
      decimals: params.decimals as any, // OpenAPI types u8 as number
      name: params.name,
    };
    const response = await this.post<AdminResponse>("/api/admin", request);
    if (response.type !== "create_token") {
      throw new ApiError("Invalid response type", 500);
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
      type: "create_market",
      base_ticker: params.baseTicker,
      quote_ticker: params.quoteTicker,
      tick_size: params.tickSize as any, // Backend expects string (u128)
      lot_size: params.lotSize as any,
      min_size: params.minSize as any,
      maker_fee_bps: params.makerFeeBps as any, // OpenAPI types i32 as number
      taker_fee_bps: params.takerFeeBps as any,
    };
    const response = await this.post<AdminResponse>("/api/admin", request);
    if (response.type !== "create_market") {
      throw new ApiError("Invalid response type", 500);
    }
    return response.market;
  }

  async adminFaucet(params: {
    userAddress: string;
    tokenTicker: string;
    amount: string;
  }): Promise<{ newBalance: string }> {
    const request: AdminRequest = {
      type: "faucet",
      user_address: params.userAddress,
      token_ticker: params.tokenTicker,
      amount: params.amount,
      signature: "admin",
    };
    const response = await this.post<AdminResponse>("/api/admin", request);
    if (response.type !== "faucet") {
      throw new ApiError("Invalid response type", 500);
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
    const response = await this.post<CandlesResponse>("/api/candles", request);
    return response.candles;
  }

  // ===== HTTP Helpers =====

  private async post<T>(path: string, body: unknown): Promise<T> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify(body),
        signal: controller.signal,
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const error = await response.json().catch(() => ({ error: response.statusText }));
        throw new ApiError(error.error || "Request failed", response.status, error.code);
      }

      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);
      if (error instanceof ApiError) {
        throw error;
      }
      throw new ApiError("Network error", 0, undefined, error);
    }
  }
}
