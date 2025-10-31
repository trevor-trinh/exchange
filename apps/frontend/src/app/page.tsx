"use client";

import { useEffect } from "react";
import { useExchangeStore } from "@/lib/store";
import { useMarkets } from "@/lib/hooks";
import { Orderbook } from "@/components/Orderbook";
import { TradingViewChart } from "@/components/TradingViewChart";
import { TradePanel } from "@/components/TradePanel";
import { BottomPanel } from "@/components/BottomPanel";

export default function Home() {
  const { markets, isLoading } = useMarkets();
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectMarket = useExchangeStore((state) => state.selectMarket);
  const selectedMarket = useExchangeStore((state) =>
    state.markets.find((m) => m.id === selectedMarketId)
  );

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
    <main className="min-h-screen bg-[#0a0a0a] text-white p-3">
      <div className="max-w-[2000px] mx-auto">
        {/* Header */}
        <div className="mb-4">
          <div className="flex items-center justify-between">
            {/* Logo and Market Info */}
            <div className="flex items-center gap-6">
              <h1 className="text-2xl font-semibold tracking-tight bg-gradient-to-r from-white to-gray-400 bg-clip-text text-transparent">
                Exchange
              </h1>

              {/* Market Selector - Pill style */}
              {isLoading ? (
                <div className="text-gray-500 text-sm">Loading...</div>
              ) : markets.length === 0 ? (
                <div className="text-gray-500 text-sm">No markets</div>
              ) : (
                <div className="flex items-center gap-3">
                  <select
                    value={selectedMarketId || ""}
                    onChange={(e) => selectMarket(e.target.value)}
                    className="bg-[#141414] border border-gray-800/50 rounded-xl px-4 py-2.5 text-white text-sm font-medium focus:outline-none focus:border-blue-500/50 hover:border-gray-700 transition-colors cursor-pointer"
                  >
                    {markets.map((market) => (
                      <option key={market.id} value={market.id}>
                        {market.base_ticker}/{market.quote_ticker}
                      </option>
                    ))}
                  </select>

                  {/* Market Stats - Optional */}
                  {selectedMarket && (
                    <div className="hidden lg:flex items-center gap-4 text-xs">
                      <div className="flex flex-col">
                        <span className="text-gray-500">Tick Size</span>
                        <span className="text-gray-300 font-medium">{selectedMarket.tick_size}</span>
                      </div>
                      <div className="flex flex-col">
                        <span className="text-gray-500">Lot Size</span>
                        <span className="text-gray-300 font-medium">{selectedMarket.lot_size}</span>
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>

            {/* Right side actions */}
            <div className="flex items-center gap-2">
              <button className="px-3 py-2 rounded-lg bg-[#141414] border border-gray-800/50 hover:border-gray-700 transition-colors text-sm text-gray-400 hover:text-white">
                Settings
              </button>
            </div>
          </div>
        </div>

        {/* Main Trading Grid */}
        <div className="grid grid-cols-12 gap-3 mb-3">
          {/* Chart - Takes up most of the space */}
          <div className="col-span-12 lg:col-span-7">
            <TradingViewChart />
          </div>

          {/* Orderbook with Trades tab */}
          <div className="col-span-12 lg:col-span-3">
            <Orderbook />
          </div>

          {/* Trade Panel */}
          <div className="col-span-12 lg:col-span-2">
            <TradePanel />
          </div>
        </div>

        {/* Bottom Panel - Balances and Orders */}
        <BottomPanel />
      </div>
    </main>
  );
}
