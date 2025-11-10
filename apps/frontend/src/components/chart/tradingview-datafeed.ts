/**
 * TradingView Datafeed Implementation
 *
 * This implements the TradingView Datafeed API to connect to our exchange backend.
 * https://www.tradingview.com/charting-library-docs/latest/connecting_data/Datafeed-API
 */

import type {
  IBasicDataFeed,
  LibrarySymbolInfo,
  ResolutionString,
  Bar,
  HistoryCallback,
  OnReadyCallback,
  ResolveCallback,
  SubscribeBarsCallback,
  SearchSymbolsCallback,
} from "../../../public/vendor/trading-view/charting_library";
import { toDisplayValue, type Candle } from "@exchange/sdk";

// ErrorCallback is not exported from TradingView types, so we define it here
type ErrorCallback = (reason: string) => void;

import { getExchangeClient } from "@/lib/api";

// Resolution mapping from TradingView to our backend
const resolutionMap: Record<string, string> = {
  "1": "1m",
  "5": "5m",
  "15": "15m",
  "60": "1h",
  D: "1d",
  "1D": "1d",
};

// Resolution to seconds mapping for candle aggregation
const resolutionToSeconds: Record<string, number> = {
  "1": 60, // 1 minute
  "5": 300, // 5 minutes
  "15": 900, // 15 minutes
  "60": 3600, // 1 hour
  D: 86400, // 1 day
  "1D": 86400,
};

interface BarSubscription {
  symbolInfo: LibrarySymbolInfo;
  resolution: ResolutionString;
  onTick: SubscribeBarsCallback;
  listenerGuid: string;
  currentBar: Bar | null;
  intervalSeconds: number;
}

export class ExchangeDatafeed implements IBasicDataFeed {
  private configurationData = {
    supported_resolutions: ["1", "5", "15", "60", "D"] as ResolutionString[],
    exchanges: [{ value: "Exchange", name: "Exchange", desc: "Exchange" }],
    symbols_types: [{ name: "crypto", value: "crypto" }],
  };
  private subscriptions = new Map<string, BarSubscription>();
  private client = getExchangeClient();
  private tradeUnsubscribers = new Map<string, () => void>();

  /**
   * Called when the library is initialized
   */
  onReady(callback: OnReadyCallback): void {
    setTimeout(() => {
      callback(this.configurationData);
    }, 0);
  }

  /**
   * Search for symbols (markets)
   */
  searchSymbols(userInput: string, _exchange: string, _symbolType: string, onResult: SearchSymbolsCallback): void {
    // Use SDK cache if available
    const markets = this.client.cache.getAllMarkets();

    const symbols = markets
      .filter((m) => m.id.toLowerCase().includes(userInput.toLowerCase()))
      .map((m) => ({
        symbol: m.id,
        full_name: m.id,
        description: `${m.base_ticker}/${m.quote_ticker}`,
        exchange: "Exchange",
        type: "crypto",
      }));
    onResult(symbols);
  }

  /**
   * Resolve symbol info
   */
  resolveSymbol(symbolName: string, onResolve: ResolveCallback, onError: ErrorCallback): void {
    // Use SDK cache directly
    const market = this.client.cache.getMarket(symbolName);

    if (!market) {
      onError("Symbol not found");
      return;
    }

    // Look up token decimals from cache
    const quoteToken = this.client.cache.getToken(market.quote_ticker);
    const baseToken = this.client.cache.getToken(market.base_ticker);

    if (!quoteToken || !baseToken) {
      onError("Token not found");
      return;
    }

    // Calculate pricescale based on quote decimals
    // Limit to 2 decimals for better readability (can show prices like 1234.56)
    // pricescale is 10^decimals (e.g., 2 decimals = 100)
    const priceDecimals = 2;
    const pricescale = Math.pow(10, priceDecimals);

    const symbolInfo: LibrarySymbolInfo = {
      name: market.id,
      ticker: market.id,
      description: `${market.base_ticker}/${market.quote_ticker}`,
      type: "crypto",
      session: "24x7",
      timezone: "Etc/UTC",
      exchange: "Exchange",
      minmov: 1,
      pricescale: pricescale,
      has_intraday: true,
      listed_exchange: "Exchange",
      has_weekly_and_monthly: false,
      supported_resolutions: this.configurationData.supported_resolutions,
      volume_precision: 2,
      data_status: "streaming",
      format: "price",
    };

    onResolve(symbolInfo);
  }

  /**
   * Fetch historical bars
   */
  getBars(
    symbolInfo: LibrarySymbolInfo,
    resolution: ResolutionString,
    periodParams: {
      from: number;
      to: number;
      firstDataRequest: boolean;
      countBack?: number;
    },
    onResult: HistoryCallback,
    onError: ErrorCallback
  ): void {
    const { from, to, countBack: _countBack } = periodParams;
    const interval = resolutionMap[resolution] || "1m";

    // Get market config from SDK cache
    const market = this.client.cache.getMarket(symbolInfo.name);
    if (!market) {
      console.error("[TradingView Datafeed] Market not found in cache");
      onError("Market not found in cache");
      return;
    }

    // Look up token decimals from SDK cache
    const quoteToken = this.client.cache.getToken(market.quote_ticker);
    const baseToken = this.client.cache.getToken(market.base_ticker);

    if (!quoteToken || !baseToken) {
      onError("Token not found in cache");
      return;
    }

    // Fetch candles using the SDK
    this.client
      .getCandles({
        marketId: symbolInfo.name,
        interval,
        from,
        to,
      })
      .then((candles: Candle[]) => {
        if (!candles || candles.length === 0) {
          onResult([], { noData: true });
          return;
        }

        const bars: Bar[] = candles.map((candle: Candle) => ({
          time: candle.timestamp * 1000, // TradingView expects milliseconds
          open: toDisplayValue(String(candle.open), quoteToken.decimals),
          high: toDisplayValue(String(candle.high), quoteToken.decimals),
          low: toDisplayValue(String(candle.low), quoteToken.decimals),
          close: toDisplayValue(String(candle.close), quoteToken.decimals),
          volume: toDisplayValue(String(candle.volume), baseToken.decimals),
        }));

        onResult(bars, { noData: false });
      })
      .catch((error: unknown) => {
        console.error("Error fetching candles:", error);
        onError("Error fetching candles");
      });
  }

  /**
   * Subscribe to real-time bar updates
   */
  subscribeBars(
    symbolInfo: LibrarySymbolInfo,
    resolution: ResolutionString,
    onTick: SubscribeBarsCallback,
    listenerGuid: string,
    _onResetCacheNeededCallback: () => void
  ): void {
    const intervalSeconds = resolutionToSeconds[resolution];
    if (!intervalSeconds) {
      console.error("Unsupported resolution:", resolution);
      return;
    }

    // Store subscription
    const subscription: BarSubscription = {
      symbolInfo,
      resolution,
      onTick,
      listenerGuid,
      currentBar: null,
      intervalSeconds,
    };
    this.subscriptions.set(listenerGuid, subscription);

    // Subscribe to trades for this market if not already subscribed
    const marketId = symbolInfo.name;
    const isFirstSubscription =
      Array.from(this.subscriptions.values()).filter((sub) => sub.symbolInfo.name === marketId).length === 1;

    if (isFirstSubscription) {
      // Subscribe using SDK convenience method (receives enhanced trades)
      const unsubscribe = this.client.onTrades(marketId, (enhancedTrade) => {
        this.handleEnhancedTrade(enhancedTrade);
      });

      this.tradeUnsubscribers.set(marketId, unsubscribe);
    }
  }

  /**
   * Unsubscribe from real-time bar updates
   */
  unsubscribeBars(listenerGuid: string): void {
    const subscription = this.subscriptions.get(listenerGuid);
    if (!subscription) {
      return;
    }

    const marketId = subscription.symbolInfo.name;
    this.subscriptions.delete(listenerGuid);

    // If no more subscriptions for this market, unsubscribe from trades
    const hasOtherSubscriptions = Array.from(this.subscriptions.values()).some(
      (sub) => sub.symbolInfo.name === marketId
    );

    if (!hasOtherSubscriptions) {
      const unsubscribe = this.tradeUnsubscribers.get(marketId);
      if (unsubscribe) {
        unsubscribe();
        this.tradeUnsubscribers.delete(marketId);
      }
    }
  }

  /**
   * Trade handler for WebSocket events
   */
  /**
   * Handle enhanced trade from SDK (WebSocket)
   */
  private handleEnhancedTrade(trade: import("@exchange/sdk").EnhancedTrade): void {
    // SDK already enhanced the trade with display values!
    const price = trade.priceValue;
    const size = trade.sizeValue;
    const timestamp = trade.timestamp.getTime(); // Already a Date object

    // Update all subscriptions for this market
    this.subscriptions.forEach((subscription) => {
      if (subscription.symbolInfo.name !== trade.market_id) {
        return;
      }

      // Calculate the bar time (start of the candle interval)
      const barTime = this.getBarTime(timestamp, subscription.intervalSeconds);

      // If this is a new bar, send the previous bar and create a new one
      if (subscription.currentBar && subscription.currentBar.time !== barTime) {
        subscription.onTick(subscription.currentBar);
        subscription.currentBar = null;
      }

      // Update or create the current bar
      if (!subscription.currentBar) {
        subscription.currentBar = {
          time: barTime,
          open: price,
          high: price,
          low: price,
          close: price,
          volume: size,
        };
      } else {
        subscription.currentBar.high = Math.max(subscription.currentBar.high, price);
        subscription.currentBar.low = Math.min(subscription.currentBar.low, price);
        subscription.currentBar.close = price;
        subscription.currentBar.volume = (subscription.currentBar.volume || 0) + size;
      }

      // Send the updated bar
      subscription.onTick(subscription.currentBar);
    });
  }

  /**
   * Calculate the start time of the bar for a given timestamp and interval
   */
  private getBarTime(timestamp: number, intervalSeconds: number): number {
    const intervalMs = intervalSeconds * 1000;
    return Math.floor(timestamp / intervalMs) * intervalMs;
  }
}
