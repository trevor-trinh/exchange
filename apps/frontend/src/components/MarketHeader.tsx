"use client";

import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { useMarkets } from "@/lib/hooks";
import { AuthButton } from "@/components/AuthButton";
import { FaucetDialog } from "@/components/FaucetDialog";
import { toDisplayValue } from "@exchange/sdk";
import { formatWithoutTrailingZeros } from "@/lib/format";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import Image from "next/image";

export function MarketHeader() {
  const { markets, isLoading } = useMarkets();
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectMarket = useExchangeStore((state) => state.selectMarket);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const recentTrades = useExchangeStore((state) => state.recentTrades);
  const currentPrice = recentTrades.length > 0 ? (recentTrades[0]?.priceValue ?? null) : null;

  // Look up tokens for the selected market using O(1) Record access
  const baseToken = selectedMarket ? tokens[selectedMarket.base_ticker] : null;
  const quoteToken = selectedMarket ? tokens[selectedMarket.quote_ticker] : null;

  return (
    <>
      {/* Header */}
      <div className="mb-4 md:mb-6">
        <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
          {/* Logo */}
          <div className="flex items-center gap-3 group select-none">
            <Image src="/logo3.png" alt="Exchange Logo" width={48} height={48} className="h-12 w-12" priority />

            <span className="text-2xl font-bold text-primary animate-pulse inline-block origin-center hover:scale-125 transition-transform duration-200 cursor-pointer relative">
              *<span className="absolute inset-0 text-primary blur-sm animate-pulse">*</span>
            </span>
          </div>

          {/* Actions */}
          <div className="flex items-center gap-2">
            <FaucetDialog />
            <AuthButton />
          </div>
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
                <SelectContent className="bg-card backdrop-blur-sm">
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
                <div className="h-3.5 w-px bg-primary/40"></div>
                <div className="flex items-center gap-1.5">
                  <span className="text-primary/60 uppercase tracking-wider font-semibold">Price</span>
                  <span className="text-primary font-mono font-bold">
                    {currentPrice !== null ? currentPrice.toFixed(2) : "—"}
                  </span>
                  <span className="text-muted-foreground/60">{selectedMarket.quote_ticker}</span>
                </div>
                <div className="h-3.5 w-px bg-primary/40"></div>
                <div className="flex items-center gap-1.5">
                  <span className="text-muted-foreground/60 uppercase tracking-wider">Tick</span>
                  <span className="text-foreground font-mono font-medium">
                    {formatWithoutTrailingZeros(
                      toDisplayValue(selectedMarket.tick_size, quoteToken.decimals),
                      Math.min(quoteToken.decimals, 8)
                    )}
                  </span>
                </div>
                <div className="h-3.5 w-px bg-primary/40"></div>
                <div className="flex items-center gap-1.5">
                  <span className="text-muted-foreground/60 uppercase tracking-wider">Lot</span>
                  <span className="text-foreground font-mono font-medium">
                    {formatWithoutTrailingZeros(
                      toDisplayValue(selectedMarket.lot_size, baseToken.decimals),
                      Math.min(baseToken.decimals, 8)
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
