"use client";

import { useExchangeStore } from "@/lib/store";
import { useMarkets } from "@/lib/hooks";
import { AuthButton } from "@/components/AuthButton";
import { toDisplayValue } from "@/lib/format";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

export function MarketHeader() {
  const { markets, isLoading } = useMarkets();
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectMarket = useExchangeStore((state) => state.selectMarket);
  const selectedMarket = useExchangeStore((state) => state.markets.find((m) => m.id === selectedMarketId));
  const tokens = useExchangeStore((state) => state.tokens);
  const currentPrice = useExchangeStore((state) => {
    if (state.priceHistory.length > 0) {
      return state.priceHistory[state.priceHistory.length - 1]?.price ?? null;
    }
    return null;
  });

  // Look up tokens for the selected market
  const baseToken = selectedMarket ? tokens.find((t) => t.ticker === selectedMarket.base_ticker) : null;
  const quoteToken = selectedMarket ? tokens.find((t) => t.ticker === selectedMarket.quote_ticker) : null;

  return (
    <>
      {/* Header */}
      <div className="mb-4 md:mb-6">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          {/* Logo */}
          <h1 className="text-2xl font-bold tracking-tight">Exchange</h1>

          {/* Auth Button */}
          <AuthButton />
        </div>
      </div>

      {/* Market Selector and Stats */}
      <div className="mb-3">
        <div className="bg-card/50 backdrop-blur-xl border border-border rounded px-3 py-1.5">
          <div className="flex items-center gap-4 text-xs overflow-x-auto">
            {/* Market Selector */}
            {isLoading ? (
              <div className="text-muted-foreground">Loading...</div>
            ) : markets.length === 0 ? (
              <div className="text-muted-foreground">No markets</div>
            ) : (
              <Select value={selectedMarketId || ""} onValueChange={selectMarket}>
                <SelectTrigger className="w-[130px] bg-primary/10 border-primary/40 hover:bg-primary/20 hover:border-primary/50 h-7 text-xs transition-colors">
                  <SelectValue placeholder="Select market" />
                </SelectTrigger>
                <SelectContent className="bg-card/100 backdrop-blur-sm">
                  {markets.map((market) => (
                    <SelectItem key={market.id} value={market.id}>
                      {market.base_ticker}/{market.quote_ticker}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            )}

            {selectedMarket && baseToken && quoteToken && (
              <>
                <div className="h-3.5 w-[1px] bg-primary/40"></div>
                <div className="flex items-center gap-1.5">
                  <span className="text-primary/60 uppercase tracking-wider font-semibold">Price</span>
                  <span className="text-primary font-mono font-bold">
                    {currentPrice !== null ? currentPrice.toFixed(2) : "—"}
                  </span>
                  <span className="text-muted-foreground/60">{selectedMarket.quote_ticker}</span>
                </div>
                <div className="h-3.5 w-[1px] bg-primary/40"></div>
                <div className="flex items-center gap-1.5">
                  <span className="text-muted-foreground/60 uppercase tracking-wider">Tick</span>
                  <span className="text-foreground font-mono font-medium">
                    {toDisplayValue(selectedMarket.tick_size, quoteToken.decimals).toFixed(
                      Math.min(quoteToken.decimals, 8),
                    )}
                  </span>
                </div>
                <div className="h-3.5 w-[1px] bg-primary/40"></div>
                <div className="flex items-center gap-1.5">
                  <span className="text-muted-foreground/60 uppercase tracking-wider">Lot</span>
                  <span className="text-foreground font-mono font-medium">
                    {toDisplayValue(selectedMarket.lot_size, baseToken.decimals).toFixed(
                      Math.min(baseToken.decimals, 8),
                    )}
                  </span>
                </div>
              </>
            )}
          </div>
        </div>
      </div>
    </>
  );
}
