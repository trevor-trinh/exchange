"use client";

import { useOrderbook, useTrades } from "@/lib/hooks";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";

export function Orderbook() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);

  // Get orderbook data with calculated values from hook
  const { spread, asksWithCumulative, bidsWithCumulative, maxAskCumulative, maxBidCumulative } =
    useOrderbook(selectedMarketId);
  const trades = useTrades(selectedMarketId);

  // Early return if no market selected
  if (!selectedMarketId || !selectedMarket) {
    return (
      <Card className="h-full min-h-[400px]">
        <CardContent className="flex items-center justify-center h-full">
          <p className="text-muted-foreground text-sm">Select a market to view orderbook</p>
        </CardContent>
      </Card>
    );
  }
  return (
    <Card className="flex flex-col h-full gap-0 py-0 overflow-hidden">
      <Tabs defaultValue="orderbook" className="flex-1 flex flex-col gap-0 min-h-0">
        <TabsList className="w-full justify-start rounded-none border-b border-border h-auto p-0 bg-card/50 backdrop-blur-sm shrink-0 z-10">
          <TabsTrigger value="orderbook" className="flex-1 rounded-none">
            Orderbook
          </TabsTrigger>
          <TabsTrigger value="trades" className="flex-1 rounded-none">
            Trades
          </TabsTrigger>
        </TabsList>
        <TabsContent
          value="orderbook"
          className="overflow-hidden flex flex-col mt-0 flex-1 min-h-0 data-[state=active]:animate-in data-[state=active]:fade-in-0 data-[state=active]:slide-in-from-bottom-1 data-[state=active]:duration-200"
        >
          <div className="flex justify-between font-medium mb-3 text-xs text-muted-foreground px-2 py-2 uppercase tracking-wide shrink-0">
            <span>Price ({selectedMarket.quote_ticker})</span>
            <span>Size ({selectedMarket.base_ticker})</span>
          </div>
          <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
            {/* Asks (Sell orders - Red) - Takes up top half, aligned to bottom */}
            <div className="flex-1 flex flex-col justify-end overflow-hidden">
              <div className="flex flex-col-reverse px-2 space-y-reverse space-y-0.5">
                {asksWithCumulative.map((ask, i) => {
                  const depthPercentage = (ask.cumulative / maxAskCumulative) * 100;
                  return (
                    <div
                      key={i}
                      className="relative flex justify-between text-sm hover:bg-red-500/20 px-2 py-0.5 transition-colors duration-0"
                    >
                      {/* Depth background */}
                      <div
                        className="absolute left-0 top-0 bottom-0 bg-red-500/10 transition-all duration-300 ease-out"
                        style={{ width: `${depthPercentage}%` }}
                      />
                      <span className="relative z-10 text-red-500 font-medium">{ask.priceDisplay}</span>
                      <span className="relative z-10 text-muted-foreground">{ask.sizeDisplay}</span>
                    </div>
                  );
                })}
              </div>
            </div>
            {/* Spread separator - Always in the middle */}
            <div className="flex items-center justify-center py-2 shrink-0">
              <div className="flex-1 border-t border-border"></div>
              <span className="px-3 text-xs text-muted-foreground font-medium whitespace-nowrap">
                SPREAD {spread.spreadPercentage}%
              </span>
              <div className="flex-1 border-t border-border"></div>
            </div>
            {/* Bids (Buy orders - Green) - Takes up bottom half, aligned to top */}
            <div className="flex-1 flex flex-col justify-start overflow-hidden">
              <div className="px-2 space-y-0.5">
                {bidsWithCumulative.map((bid, i) => {
                  const depthPercentage = (bid.cumulative / maxBidCumulative) * 100;
                  return (
                    <div
                      key={i}
                      className="relative flex justify-between text-sm hover:bg-green-500/20 px-2 py-0.5 transition-colors duration-0"
                    >
                      {/* Depth background */}
                      <div
                        className="absolute left-0 top-0 bottom-0 bg-green-500/10 transition-all duration-300 ease-out"
                        style={{ width: `${depthPercentage}%` }}
                      />
                      <span className="relative z-10 text-green-500 font-medium">{bid.priceDisplay}</span>
                      <span className="relative z-10 text-muted-foreground">{bid.sizeDisplay}</span>
                    </div>
                  );
                })}
              </div>
            </div>
          </div>
        </TabsContent>
        <TabsContent
          value="trades"
          className="flex-1 overflow-hidden flex flex-col mt-0 min-h-0 data-[state=active]:animate-in data-[state=active]:fade-in-0 data-[state=active]:slide-in-from-bottom-1 data-[state=active]:duration-200"
        >
          <div className="flex justify-between font-medium mb-3 text-xs text-muted-foreground px-2 py-2 uppercase tracking-wide shrink-0">
            <span>Price</span>
            <span>Size</span>
            <span>Time</span>
          </div>
          <div className="flex-1 overflow-y-auto min-h-0 overflow-hidden">
            {trades.length === 0 ? (
              <p className="text-muted-foreground text-sm px-2">No recent trades</p>
            ) : (
              <div className="px-2 space-y-0.5">
                {trades.slice(0, 50).map((trade) => {
                  // Use trade side to determine color (buy = green, sell = red)
                  const isBuy = trade.side === "buy";
                  const isSell = trade.side === "sell";
                  return (
                    <div
                      key={trade.id}
                      className="flex justify-between text-sm hover:bg-muted/50 px-2 py-0.5 transition-colors duration-0"
                    >
                      <span
                        className={`font-medium ${
                          isBuy ? "text-green-500" : isSell ? "text-red-500" : "text-foreground"
                        }`}
                      >
                        {trade.priceDisplay}
                      </span>
                      <span className="text-muted-foreground">{trade.sizeDisplay}</span>
                      <span className="text-muted-foreground text-xs">
                        {trade.timestamp instanceof Date
                          ? trade.timestamp.toLocaleTimeString()
                          : new Date(trade.timestamp).toLocaleTimeString()}
                      </span>
                    </div>
                  );
                })}
              </div>
            )}
          </div>
        </TabsContent>
      </Tabs>
    </Card>
  );
}
