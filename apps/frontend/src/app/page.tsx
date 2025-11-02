"use client";

import { useEffect } from "react";
import { useExchangeStore } from "@/lib/store";
import { useMarkets } from "@/lib/hooks";
import { Orderbook } from "@/components/Orderbook";
import { TradingViewChart } from "@/components/TradingViewChart";
import { TradePanel } from "@/components/TradePanel";
import { BottomPanel } from "@/components/BottomPanel";
import { MarketHeader } from "@/components/MarketHeader";

export default function Home() {
  const { markets } = useMarkets();
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectMarket = useExchangeStore((state) => state.selectMarket);

  // Auto-select BTC/USDC market when markets load
  useEffect(() => {
    console.log("[Home] useEffect - markets:", markets.length, "selectedMarketId:", selectedMarketId);
    if (markets.length > 0 && !selectedMarketId) {
      const btcUsdcMarket = markets.find((m) => m.id === "BTC/USDC");
      if (btcUsdcMarket) {
        console.log("[Home] Auto-selecting BTC/USDC market");
        selectMarket(btcUsdcMarket.id);
      } else {
        console.log("[Home] Auto-selecting first market:", markets[0]?.id);
        selectMarket(markets[0]?.id || "");
      }
    }
  }, [markets, selectedMarketId, selectMarket]);

  return (
    <main className="min-h-screen bg-background text-foreground p-4 dither-strong">
      <div className="max-w-[2000px] mx-auto">
        <MarketHeader />

        {/* Main Trading Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-12 gap-4 mb-4">
          {/* Chart - Takes up most of the space */}
          <div className="md:col-span-2 lg:col-span-8 h-[500px] md:h-[600px]">
            <TradingViewChart />
          </div>

          {/* Orderbook with Trades tab */}
          <div className="md:col-span-1 lg:col-span-2 h-[500px] md:h-[600px]">
            <Orderbook />
          </div>

          {/* Trade Panel */}
          <div className="md:col-span-1 lg:col-span-2 h-[500px] md:h-[600px]">
            <TradePanel />
          </div>
        </div>

        {/* Bottom Panel - Balances and Orders */}
        <BottomPanel />
      </div>
    </main>
  );
}
