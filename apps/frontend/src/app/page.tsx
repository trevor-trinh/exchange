"use client";

import { useEffect } from "react";
import { useExchangeStore } from "@/lib/store";
import { useMarkets } from "@/lib/hooks";
import { Orderbook } from "@/components/Orderbook";
import { TradingViewChart } from "@/components/TradingViewChart";
import { TradePanel } from "@/components/TradePanel";
import { BottomPanel } from "@/components/BottomPanel";
import { formatPrice, formatSize } from "@/lib/format";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";

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
    <main className="min-h-screen bg-background text-foreground p-4">
      <div className="max-w-[2000px] mx-auto">
        {/* Header */}
        <div className="mb-4">
          <div className="flex items-center justify-between">
            {/* Logo and Market Info */}
            <div className="flex items-center gap-6">
              <h1 className="text-2xl font-bold tracking-tight">Exchange</h1>

              {/* Market Selector */}
              {isLoading ? (
                <div className="text-muted-foreground text-sm">Loading...</div>
              ) : markets.length === 0 ? (
                <div className="text-muted-foreground text-sm">No markets</div>
              ) : (
                <div className="flex items-center gap-4">
                  <Select
                    value={selectedMarketId || ""}
                    onValueChange={selectMarket}
                  >
                    <SelectTrigger className="w-[180px]">
                      <SelectValue placeholder="Select market" />
                    </SelectTrigger>
                    <SelectContent>
                      {markets.map((market) => (
                        <SelectItem key={market.id} value={market.id}>
                          {market.base_ticker}/{market.quote_ticker}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>

                  {/* Market Stats */}
                  {selectedMarket && (
                    <div className="hidden lg:flex items-center gap-6 text-xs">
                      <div className="flex flex-col gap-1">
                        <span className="text-muted-foreground font-medium">Tick Size</span>
                        <span className="text-foreground font-mono">
                          {formatPrice(selectedMarket.tick_size, selectedMarket.quote_decimals)} {selectedMarket.quote_ticker}
                        </span>
                      </div>
                      <div className="flex flex-col gap-1">
                        <span className="text-muted-foreground font-medium">Lot Size</span>
                        <span className="text-foreground font-mono">
                          {formatSize(selectedMarket.lot_size, selectedMarket.base_decimals)} {selectedMarket.base_ticker}
                        </span>
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>

          </div>
        </div>

        {/* Main Trading Grid */}
        <div className="grid grid-cols-12 gap-4 mb-4">
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
