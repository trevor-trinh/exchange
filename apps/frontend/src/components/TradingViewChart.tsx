"use client";

import { useEffect, useRef } from "react";
import { useExchangeStore } from "@/lib/store";
import { ExchangeDatafeed } from "@/lib/tradingview-datafeed";

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore - TradingView types
import type {
  IChartingLibraryWidget,
  ChartingLibraryWidgetOptions,
  ResolutionString,
} from "../../public/vendor/trading-view/charting_library";

// Extend window to include TradingView
declare global {
  interface Window {
    TradingView?: {
      widget: new (options: ChartingLibraryWidgetOptions) => IChartingLibraryWidget;
    };
  }
}

export function TradingViewChart() {
  const containerRef = useRef<HTMLDivElement>(null);
  const widgetRef = useRef<IChartingLibraryWidget | null>(null);
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);

  useEffect(() => {
    if (!containerRef.current || !selectedMarketId) return;

    // Check if TradingView library is loaded
    if (typeof window === "undefined" || !window.TradingView) {
      console.error("TradingView library not loaded");
      return;
    }

    const TradingView = window.TradingView;

    const widgetOptions: ChartingLibraryWidgetOptions = {
      symbol: selectedMarketId,
      datafeed: new ExchangeDatafeed(),
      interval: "1" as ResolutionString, // 1 minute
      container: containerRef.current,
      library_path: "/vendor/trading-view/",
      locale: "en",
      disabled_features: ["use_localstorage_for_settings", "volume_force_overlay"],
      enabled_features: ["study_templates"],
      fullscreen: false,
      autosize: true,
      theme: "dark",
      custom_css_url: undefined,
      settings_overrides: {
        // Background
        "paneProperties.background": "#000000",
        "paneProperties.backgroundType": "solid",
        "paneProperties.backgroundGradientStartColor": "#000000",
        "paneProperties.backgroundGradientEndColor": "#000000",

        // Chart style - 1 for candles
        "mainSeriesProperties.style": 1,

        // Candle colors
        "mainSeriesProperties.candleStyle.upColor": "#02B36B",
        "mainSeriesProperties.candleStyle.downColor": "#E23659",
        "mainSeriesProperties.candleStyle.wickUpColor": "#02B36B",
        "mainSeriesProperties.candleStyle.wickDownColor": "#E23659",
        "mainSeriesProperties.candleStyle.borderUpColor": "#02B36B",
        "mainSeriesProperties.candleStyle.borderDownColor": "#E23659",
        "mainSeriesProperties.candleStyle.drawWick": true,
        "mainSeriesProperties.candleStyle.drawBorder": true,

        // Line chart colors (fallback)
        "mainSeriesProperties.lineStyle.color": "#807ADB",
        "mainSeriesProperties.lineStyle.linewidth": 1.5,
      },
      studies_overrides: {
        "volume.volume.color.0": "#E23659",
        "volume.volume.color.1": "#02B36B",
        "volume.volume.transparency": 65,
      },
    };

    try {
      const widget = new TradingView.widget(widgetOptions);
      widgetRef.current = widget;

      widget.onChartReady(() => {
        console.log("TradingView chart is ready");
      });
    } catch (error) {
      console.error("Failed to create TradingView widget:", error);
    }

    // Cleanup
    return () => {
      if (widgetRef.current) {
        widgetRef.current.remove();
        widgetRef.current = null;
      }
    };
  }, [selectedMarketId]);

  if (!selectedMarketId) {
    return (
      <div className="p-4 border rounded flex items-center justify-center h-96">
        <p className="text-gray-500">Select a market to view chart</p>
      </div>
    );
  }

  return (
    <div className="border rounded overflow-hidden" style={{ height: "500px" }}>
      <div ref={containerRef} className="h-full" />
    </div>
  );
}
