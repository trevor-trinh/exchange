"use client";

import { useEffect, useRef, useState } from "react";
import { useExchangeStore } from "@/lib/store";
import { useOrders } from "@/lib/hooks/useOrders";
import { ExchangeDatafeed } from "@/lib/tradingview-datafeed";
import { Card, CardContent } from "@/components/ui/card";
import { useOrderLines } from "./useOrderLines";
import { getChartConfig } from "./chartConfig";

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
import type {
  IChartingLibraryWidget,
  ChartingLibraryWidgetOptions,
} from "../../../public/vendor/trading-view/charting_library";

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
  const [isChartReady, setIsChartReady] = useState(false);

  // Fetch and subscribe to orders
  useOrders();

  // Manage order lines overlay
  useOrderLines(widgetRef, isChartReady);

  useEffect(() => {
    if (!containerRef.current || !selectedMarketId) {
      return;
    }

    if (typeof window === "undefined" || !window.TradingView) {
      console.error("[TradingView] Library not loaded");
      return;
    }

    const TradingView = window.TradingView;

    // Create datafeed once and reuse
    if (!datafeedRef.current) {
      datafeedRef.current = new ExchangeDatafeed();
    }

    const widgetOptions = getChartConfig(selectedMarketId, datafeedRef.current, containerRef.current);

    try {
      const widget = new TradingView.widget(widgetOptions);
      widgetRef.current = widget;

      widget.onChartReady(() => {
        widget.activeChart().setChartType(1); // Force candlestick
        setIsChartReady(true);
      });
    } catch (error) {
      console.error("[TradingView] Failed to create widget:", error);
    }

    return () => {
      if (widgetRef.current) {
        widgetRef.current.remove();
        widgetRef.current = null;
      }
      setIsChartReady(false);
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
