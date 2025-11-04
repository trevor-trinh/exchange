"use client";

import { useEffect, useRef, useState } from "react";
import { useExchangeStore } from "@/lib/store";
import { useExchangeClient } from "@/lib/hooks/useExchangeClient";
import { useOrders } from "@/lib/hooks/useOrders";
import { ExchangeDatafeed } from "@/lib/tradingview-datafeed";
import { Card, CardContent } from "@/components/ui/card";

// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore - TradingView types
import type {
  IChartingLibraryWidget,
  ChartingLibraryWidgetOptions,
  ResolutionString,
  IOrderLineAdapter,
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
  const userAddress = useExchangeStore((state) => state.userAddress);
  const client = useExchangeClient();

  // Track when chart is ready for drawing overlays
  const [isChartReady, setIsChartReady] = useState(false);

  // Track created order lines for cleanup
  const orderLinesRef = useRef<Map<string, IOrderLineAdapter>>(new Map());

  // Fetch and subscribe to user orders (this ensures orders are always loaded)
  useOrders();

  // Get orders from store (populated by useOrders hook above)
  const userOrders = useExchangeStore((state) => state.userOrders);

  useEffect(() => {
    if (!containerRef.current || !selectedMarketId) {
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
      datafeedRef.current = new ExchangeDatafeed();
    }

    const widgetOptions: ChartingLibraryWidgetOptions = {
      symbol: selectedMarketId,
      datafeed: datafeedRef.current,
      interval: "1" as ResolutionString, // 1 minute
      container: containerRef.current,
      library_path: "/vendor/trading-view/",
      locale: "en",
      disabled_features: [
        "use_localstorage_for_settings",
        "volume_force_overlay",
        "header_symbol_search",
        "symbol_search_hot_key",
      ],
      enabled_features: ["study_templates", "side_toolbar_in_fullscreen_mode"],
      fullscreen: false,
      autosize: true,
      theme: "Dark",
      custom_css_url: "/tradingview-custom.css",
      loading_screen: {
        backgroundColor: "#1a1a1a",
        foregroundColor: "#9d7efa",
      },
      // Customize trading primitives
      trading_customization: {
        position: {
          lineColor: "#9d7efa",
          lineWidth: 2,
          bodyBorderColor: "#9d7efa",
          bodyBackgroundColor: "rgba(157, 126, 250, 0.15)",
          bodyTextColor: "#e9d5ff",
        },
        order: {
          lineColor: "#a295c1",
          lineWidth: 2,
          bodyBorderColor: "#9d7efa",
          bodyBackgroundColor: "rgba(157, 126, 250, 0.1)",
          bodyTextColor: "#e9d5ff",
          cancelButtonBorderColor: "#ef4444",
          cancelButtonBackgroundColor: "rgba(239, 68, 68, 0.15)",
          cancelButtonIconColor: "#ef4444",
        },
      },
      settings_overrides: {
        // Background - Simple gray
        "paneProperties.background": "#1a1a1a",
        "paneProperties.backgroundType": "solid",

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

        // Scale background
        "scalesProperties.backgroundColor": "#1a1a1a",
      },
      studies_overrides: {
        "volume.volume.color.0": "#ef4444",
        "volume.volume.color.1": "#22c55e",
        "volume.volume.transparency": 65,
        // Moving averages
        "volume.volume ma.color": "#9d7efa",
        "volume.volume ma.linewidth": 1,
        "volume.volume ma.transparency": 30,
      },
      // Override chart watermark
      time_frames: [
        { text: "1m", resolution: "1" as ResolutionString, description: "1 Minute" },
        { text: "5m", resolution: "5" as ResolutionString, description: "5 Minutes" },
        { text: "15m", resolution: "15" as ResolutionString, description: "15 Minutes" },
        { text: "1h", resolution: "60" as ResolutionString, description: "1 Hour" },
        { text: "4h", resolution: "240" as ResolutionString, description: "4 Hours" },
        { text: "1D", resolution: "D" as ResolutionString, description: "1 Day" },
      ],
    };

    try {
      const widget = new TradingView.widget(widgetOptions);
      widgetRef.current = widget;

      widget.onChartReady(() => {
        // Force candlestick chart style
        widget.activeChart().setChartType(1); // 1 = Candles

        // Signal that chart is ready for overlays
        setIsChartReady(true);
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
      // Clear all order lines
      orderLinesRef.current.clear();
      // Reset chart ready state
      setIsChartReady(false);
    };
  }, [selectedMarketId]);

  // Manage order lines for open limit orders
  useEffect(() => {
    if (!widgetRef.current || !selectedMarketId || !isChartReady) return;

    const chart = widgetRef.current.activeChart();
    if (!chart) return;

    // Filter orders for current market that are open limit orders
    const marketOrders = userOrders.filter(
      (order) =>
        order.market_id === selectedMarketId &&
        (order.status === "pending" || order.status === "partially_filled") &&
        order.order_type === "limit"
    );

    // Remove lines for orders that no longer exist or are not open
    for (const [orderId, line] of orderLinesRef.current) {
      if (!marketOrders.find((o) => o.id === orderId)) {
        try {
          line.remove();
        } catch (err) {
          console.warn("[TradingView] Failed to remove order line:", err);
        }
        orderLinesRef.current.delete(orderId);
      }
    }

    // Create lines for new orders
    for (const order of marketOrders) {
      if (!orderLinesRef.current.has(order.id)) {
        try {
          const isBuy = order.side === "buy";
          const lineColor = isBuy ? "#22c55e" : "#ef4444";
          const bgColor = isBuy ? "rgba(34, 197, 94, 0.12)" : "rgba(239, 68, 68, 0.12)";
          const sideText = isBuy ? "BUY" : "SELL";

          const line = chart.createOrderLine();
          line
            .setPrice(order.priceValue)
            .setText(`${sideText} ${order.priceDisplay} × ${order.sizeDisplay}`)
            .setQuantity(order.sizeDisplay)
            .setLineColor(lineColor)
            .setLineWidth(2)
            .setBodyBorderColor(lineColor)
            .setBodyBackgroundColor(bgColor)
            .setBodyTextColor(isBuy ? "#bbf7d0" : "#fecaca")
            .setQuantityBorderColor(lineColor)
            .setQuantityBackgroundColor(bgColor)
            .setQuantityTextColor(isBuy ? "#bbf7d0" : "#fecaca")
            .setCancelButtonBorderColor("#ef4444")
            .setCancelButtonBackgroundColor("rgba(239, 68, 68, 0.2)")
            .setCancelButtonIconColor("#ef4444")
            .setCancelTooltip("Cancel Order")
            .setTooltip(`${sideText} Order: ${order.sizeDisplay} @ ${order.priceDisplay}`)
            .onCancel(async () => {
              if (!userAddress) {
                console.warn("[TradingView] Cannot cancel order: user not authenticated");
                return;
              }

              try {
                await client.cancelOrder({
                  userAddress,
                  orderId: order.id,
                  signature: "0x", // Placeholder - need wallet integration
                });
                console.log("[TradingView] Order cancelled:", order.id);
              } catch (err) {
                console.error("[TradingView] Failed to cancel order:", err);
              }
            });

          orderLinesRef.current.set(order.id, line);
        } catch (err) {
          console.warn("[TradingView] Failed to create order line:", err);
        }
      }
    }
  }, [userOrders, selectedMarketId, isChartReady, userAddress, client]);

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
