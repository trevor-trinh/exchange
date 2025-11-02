"use client";

import { useEffect, useRef } from "react";
import { useExchangeStore } from "@/lib/store";
import { ExchangeDatafeed } from "@/lib/tradingview-datafeed";
import { Card, CardContent } from "@/components/ui/card";

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
  const datafeedRef = useRef<ExchangeDatafeed | null>(null);
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);

  useEffect(() => {
    console.log('[TradingView] useEffect triggered - selectedMarketId:', selectedMarketId);
    if (!containerRef.current || !selectedMarketId) {
      console.log('[TradingView] Early return - no container or marketId');
      return;
    }

    // Check if TradingView library is loaded
    if (typeof window === "undefined" || !window.TradingView) {
      console.error("TradingView library not loaded");
      return;
    }

    const TradingView = window.TradingView;

    // Create datafeed once and reuse
    if (!datafeedRef.current) {
      console.log('[TradingView] Creating datafeed');
      datafeedRef.current = new ExchangeDatafeed();
    }

    console.log('[TradingView] Initializing chart for market:', selectedMarketId);
    console.log('[TradingView] TradingView object:', typeof TradingView, Object.keys(TradingView));

    const widgetOptions: ChartingLibraryWidgetOptions = {
      symbol: selectedMarketId,
      datafeed: datafeedRef.current,
      interval: "1" as ResolutionString, // 1 minute
      container: containerRef.current,
      library_path: "/vendor/trading-view/",
      locale: "en",
      disabled_features: ["use_localstorage_for_settings", "volume_force_overlay"],
      enabled_features: ["study_templates"],
      fullscreen: false,
      autosize: true,
      theme: "dark",
      custom_css_url: "/tradingview-custom.css",
      loading_screen: {
        backgroundColor: "#0d0a14",
        foregroundColor: "#9d7efa",
      },
      settings_overrides: {
        // Background - Match card background (hsl(260, 30%, 8%))
        "paneProperties.background": "#0d0a14",
        "paneProperties.backgroundType": "solid",
        "paneProperties.backgroundGradientStartColor": "#0d0a14",
        "paneProperties.backgroundGradientEndColor": "#0d0a14",

        // Grid lines - match border color (hsl(0, 0%, 20%))
        "paneProperties.vertGridProperties.color": "rgba(51, 51, 51, 0.3)",
        "paneProperties.horzGridProperties.color": "rgba(51, 51, 51, 0.3)",
        "paneProperties.vertGridProperties.style": 0,
        "paneProperties.horzGridProperties.style": 0,

        // Separators
        "paneProperties.separatorColor": "#333333",

        // Chart style - 1 for candles
        "mainSeriesProperties.style": 1,

        // Candle colors - vibrant green/red with glow effect
        "mainSeriesProperties.candleStyle.upColor": "#22c55e",
        "mainSeriesProperties.candleStyle.downColor": "#ef4444",
        "mainSeriesProperties.candleStyle.wickUpColor": "#22c55e",
        "mainSeriesProperties.candleStyle.wickDownColor": "#ef4444",
        "mainSeriesProperties.candleStyle.borderUpColor": "#22c55e",
        "mainSeriesProperties.candleStyle.borderDownColor": "#ef4444",
        "mainSeriesProperties.candleStyle.drawWick": true,
        "mainSeriesProperties.candleStyle.drawBorder": true,

        // Line chart colors (fallback) - primary purple
        "mainSeriesProperties.lineStyle.color": "#9d7efa",
        "mainSeriesProperties.lineStyle.linewidth": 2,

        // Area chart colors
        "mainSeriesProperties.areaStyle.color1": "rgba(157, 126, 250, 0.3)",
        "mainSeriesProperties.areaStyle.color2": "rgba(157, 126, 250, 0.05)",
        "mainSeriesProperties.areaStyle.linecolor": "#9d7efa",

        // Crosshair - purple primary
        "crosshairProperties.color": "#9d7efa",
        "crosshairProperties.width": 1,
        "crosshairProperties.style": 2,

        // Scale text color and background - match muted foreground
        "scalesProperties.textColor": "#a295c1",
        "scalesProperties.backgroundColor": "#0d0a14",
        "scalesProperties.lineColor": "#333333",
      },
      studies_overrides: {
        "volume.volume.color.0": "#ef4444",
        "volume.volume.color.1": "#22c55e",
        "volume.volume.transparency": 70,
      },
    };

    console.log('[TradingView] Creating widget...');
    try {
      const widget = new TradingView.widget(widgetOptions);
      console.log('[TradingView] Widget created successfully');
      widgetRef.current = widget;

      widget.onChartReady(() => {
        console.log("[TradingView] Chart is ready!");

        // Force candlestick chart style
        widget.activeChart().setChartType(1); // 1 = Candles
      });
    } catch (error) {
      console.error("[TradingView] Failed to create widget:", error);
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
      <Card className="flex items-center justify-center h-full min-h-[400px]">
        <CardContent className="p-6">
          <p className="text-gray-500 text-sm">Select a market to view chart</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="overflow-hidden h-full p-0 dither">
      <div ref={containerRef} className="h-full w-full" />
    </Card>
  );
}
