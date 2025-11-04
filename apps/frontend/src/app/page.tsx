"use client";

import { useEffect } from "react";
import { useExchangeStore } from "@/lib/store";
import { useMarkets } from "@/lib/hooks";
import { Orderbook } from "@/components/orderbook-panel";
import { TradingViewChart } from "@/components/TradingViewChart";
import { TradePanel } from "@/components/trade-panel";
import { BottomPanel } from "@/components/bottom-panel";
import { MarketHeader } from "@/components/MarketHeader";

export default function Home() {
  const { markets } = useMarkets();
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectMarket = useExchangeStore((state) => state.selectMarket);

  // Auto-select BTC/USDC market when markets load
  useEffect(() => {
    if (markets.length > 0 && !selectedMarketId) {
      const btcUsdcMarket = markets.find((m) => m.id === "BTC/USDC");
      if (btcUsdcMarket) {
        selectMarket(btcUsdcMarket.id);
      } else {
        selectMarket(markets[0]?.id || "");
      }
    }
  }, [markets, selectedMarketId, selectMarket]);

  return (
    <main className="min-h-screen bg-background text-foreground p-4 dither-strong">
      <div className="max-w-[2000px] mx-auto">
        <MarketHeader />

        {/* Main Trading Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-12 gap-4 mb-4">
          {/* Chart - Takes up most of the space */}
          <div className="lg:col-span-8 h-[500px] lg:h-[600px]">
            <TradingViewChart />
          </div>

          {/* Orderbook with Trades tab */}
          <div className="lg:col-span-2 h-[500px] lg:h-[600px] min-w-0">
            <Orderbook />
          </div>

          {/* Trade Panel */}
          <div className="lg:col-span-2 h-[500px] lg:h-[600px] min-w-0">
            <TradePanel />
          </div>
        </div>

        {/* Bottom Panel - Balances and Orders */}
        <BottomPanel />
      </div>
    </main>
  );
}
