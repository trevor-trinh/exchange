"use client";

import { useEffect } from "react";
import { useExchangeStore } from "@/lib/store";
import { useMarkets } from "@/lib/hooks";
import { Orderbook } from "@/components/Orderbook";
import { TradingViewChart } from "@/components/TradingViewChart";
import { TradeHistory } from "@/components/TradeHistory";

export default function Home() {
  const { markets, isLoading } = useMarkets();
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
    <main className="min-h-screen bg-black text-white p-8">
      <div className="max-w-7xl mx-auto">
        <div className="mb-8">
          <h1 className="text-4xl font-bold mb-4">Exchange Monitor</h1>

          {/* Market Selector */}
          <div className="flex items-center gap-4">
            <label className="text-sm text-gray-400">Market:</label>
            {isLoading ? (
              <div className="text-gray-500">Loading markets...</div>
            ) : (
              <select
                value={selectedMarketId || ""}
                onChange={(e) => selectMarket(e.target.value)}
                className="bg-gray-900 border border-gray-700 rounded px-4 py-2"
              >
                {markets.map((market) => (
                  <option key={market.id} value={market.id}>
                    {market.base_ticker}/{market.quote_ticker}
                  </option>
                ))}
              </select>
            )}
          </div>
        </div>

        {/* Main Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Chart - takes 2 columns */}
          <div className="lg:col-span-2">
            <TradingViewChart />
          </div>

          {/* Orderbook */}
          <div>
            <Orderbook />
          </div>

          {/* Trade History - full width below */}
          <div className="lg:col-span-3">
            <TradeHistory />
          </div>
        </div>
      </div>
    </main>
  );
}
