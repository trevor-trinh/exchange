/**
 * TradingView Datafeed Implementation
 *
 * This implements the TradingView Datafeed API to connect to our exchange backend.
 * https://www.tradingview.com/charting-library-docs/latest/connecting_data/Datafeed-API
 */

// @ts-expect-error - TradingView types
import type {
  IBasicDataFeed,
  LibrarySymbolInfo,
  ResolutionString,
  Bar,
  HistoryCallback,
  OnReadyCallback,
  ResolveCallback,
  // @ts-expect-error - TradingView types
  ErrorCallback,
  SubscribeBarsCallback,
  SearchSymbolsCallback,
} from "../../public/vendor/trading-view/charting_library";

import { exchange } from "./api";

// Resolution mapping from TradingView to our backend
const resolutionMap: Record<string, string> = {
  "1": "1m",
  "5": "5m",
  "15": "15m",
  "60": "1h",
  D: "1d",
  "1D": "1d",
};

interface Candle {
  timestamp: number;
  open: number;
  high: number;
  low: number;
  close: number;
  volume: number;
}

export class ExchangeDatafeed implements IBasicDataFeed {
  private configurationData = {
    supported_resolutions: ["1", "5", "15", "60", "D"] as ResolutionString[],
    exchanges: [{ value: "Exchange", name: "Exchange", desc: "Exchange" }],
    symbols_types: [{ name: "crypto", value: "crypto" }],
  };

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
    exchange
      .getMarkets()
      .then((markets) => {
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
      })
      .catch((error) => {
        console.error("Error searching symbols:", error);
        onResult([]);
      });
  }

  /**
   * Resolve symbol info
   */
  resolveSymbol(symbolName: string, onResolve: ResolveCallback, onError: ErrorCallback): void {
    exchange
      .getMarkets()
      .then((markets) => {
        const market = markets.find((m) => m.id === symbolName);

        if (!market) {
          onError("Symbol not found");
          return;
        }

        const symbolInfo: LibrarySymbolInfo = {
          name: market.id,
          ticker: market.id,
          description: `${market.base_ticker}/${market.quote_ticker}`,
          type: "crypto",
          session: "24x7",
          timezone: "Etc/UTC",
          exchange: "Exchange",
          minmov: 1,
          pricescale: 100, // 2 decimal places
          has_intraday: true,
          listed_exchange: "Exchange",
          has_weekly_and_monthly: false,
          supported_resolutions: this.configurationData.supported_resolutions,
          volume_precision: 8,
          data_status: "streaming",
          format: "price",
        };

        onResolve(symbolInfo);
      })
      .catch((error) => {
        console.error("Error resolving symbol:", error);
        onError("Error resolving symbol");
      });
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
    onError: ErrorCallback,
  ): void {
    const { from, to } = periodParams;
    const interval = resolutionMap[resolution] || "1m";

    // Fetch candles from our API
    fetch(
      `${process.env.NEXT_PUBLIC_API_URL}/api/candles?market_id=${symbolInfo.name}&interval=${interval}&from=${from}&to=${to}`,
    )
      .then(async (response) => {
        if (!response.ok) {
          console.error("Candles API error:", response.status, response.statusText);
          const text = await response.text();
          console.error("Error details:", text);
          throw new Error(`HTTP ${response.status}: ${text}`);
        }
        return response.json();
      })
      .then((data: { candles: Candle[] }) => {
        if (!data.candles || data.candles.length === 0) {
          onResult([], { noData: true });
          return;
        }

        const bars: Bar[] = data.candles.map((candle) => ({
          time: candle.timestamp * 1000, // TradingView expects milliseconds
          open: Number(candle.open) / 1e18, // Convert from fixed-point
          high: Number(candle.high) / 1e18,
          low: Number(candle.low) / 1e18,
          close: Number(candle.close) / 1e18,
          volume: Number(candle.volume) / 1e18,
        }));

        onResult(bars, { noData: false });
      })
      .catch((error) => {
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
    _onTick: SubscribeBarsCallback,
    listenerGuid: string,
    _onResetCacheNeededCallback: () => void,
  ): void {
    // For now, we'll use the trades WebSocket to update bars in real-time
    // This would be implemented by subscribing to trade updates and aggregating them
    console.log("subscribeBars called", {
      symbolInfo: symbolInfo.name,
      resolution,
      listenerGuid,
    });
  }

  /**
   * Unsubscribe from real-time bar updates
   */
  unsubscribeBars(listenerGuid: string): void {
    console.log("unsubscribeBars called", listenerGuid);
  }
}
