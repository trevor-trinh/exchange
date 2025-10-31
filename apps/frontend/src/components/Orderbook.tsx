"use client";

import { useOrderbook, useTrades } from "@/lib/hooks";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { formatPrice, formatSize } from "@/lib/format";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";

export function Orderbook() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const { bids, asks } = useOrderbook(selectedMarketId);
  const trades = useTrades(selectedMarketId);

  // Determine if trade is buy or sell based on price movement
  const getTradeDirection = (price: string, index: number) => {
    if (index >= trades.length - 1) return "neutral";
    const prevPrice = trades[index + 1].price;
    return parseFloat(price) >= parseFloat(prevPrice) ? "buy" : "sell";
  };

  if (!selectedMarketId || !selectedMarket) {
    return (
      <Card className="h-full">
        <CardContent className="flex items-center justify-center h-full">
          <p className="text-muted-foreground text-sm">Select a market to view orderbook</p>
        </CardContent>
      </Card>
    );
  }

  // Look up token decimals
  const baseToken = tokens.find((t) => t.ticker === selectedMarket.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket.quote_ticker);

  if (!baseToken || !quoteToken) {
    return (
      <Card className="h-full">
        <CardContent className="flex items-center justify-center h-full">
          <p className="text-muted-foreground text-sm">Loading token information...</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="flex flex-col h-full gap-0 py-0">
      <Tabs defaultValue="orderbook" className="flex-1 flex flex-col gap-0">
        <TabsList className="w-full justify-start rounded-none border-b h-auto p-0 bg-transparent">
          <TabsTrigger value="orderbook" className="flex-1 rounded-none border-b-2 border-transparent data-[state=active]:border-primary">
            Orderbook
          </TabsTrigger>
          <TabsTrigger value="trades" className="flex-1 rounded-none border-b-2 border-transparent data-[state=active]:border-primary">
            Trades
          </TabsTrigger>
        </TabsList>

        <TabsContent value="orderbook" className="overflow-hidden flex flex-col px-6 mt-0">
          <div className="flex justify-between font-medium mb-3 text-xs text-muted-foreground px-2 uppercase tracking-wide">
            <span>Price ({quoteToken.ticker})</span>
            <span>Size ({baseToken.ticker})</span>
          </div>

          <div className="h-[500px] flex flex-col justify-center">
            {/* Asks (Sell orders - Red) - Reversed so lowest ask is at bottom */}
            <div className="flex flex-col-reverse px-2 space-y-reverse space-y-0.5">
              {asks.slice(0, 10).map((ask, i) => (
                <div key={i} className="flex justify-between text-sm hover:bg-red-500/10 px-2 py-1 transition-colors">
                  <span className="text-red-500 font-medium">{formatPrice(ask.price, quoteToken.decimals)}</span>
                  <span className="text-muted-foreground">{formatSize(ask.size, baseToken.decimals)}</span>
                </div>
              ))}
            </div>

            {/* Spread separator - Centered */}
            <div className="flex items-center justify-center my-3">
              <div className="flex-1 border-t border-border"></div>
              <span className="px-3 text-xs text-muted-foreground font-medium">SPREAD</span>
              <div className="flex-1 border-t border-border"></div>
            </div>

            {/* Bids (Buy orders - Green) */}
            <div className="px-2 space-y-0.5">
              {bids.slice(0, 10).map((bid, i) => (
                <div key={i} className="flex justify-between text-sm hover:bg-green-500/10 px-2 py-1 transition-colors">
                  <span className="text-green-500 font-medium">{formatPrice(bid.price, quoteToken.decimals)}</span>
                  <span className="text-muted-foreground">{formatSize(bid.size, baseToken.decimals)}</span>
                </div>
              ))}
            </div>
          </div>
        </TabsContent>

        <TabsContent value="trades" className="flex-1 overflow-hidden flex flex-col px-6 mt-0">
          <div className="flex justify-between font-medium mb-3 text-xs text-muted-foreground px-2 uppercase tracking-wide">
            <span>Price</span>
            <span>Size</span>
            <span>Time</span>
          </div>

          <div className="h-[500px] overflow-y-auto">
            {trades.length === 0 ? (
              <p className="text-muted-foreground text-sm px-2">No recent trades</p>
            ) : (
              <div className="px-2 space-y-0.5">
                {trades.slice(0, 50).map((trade, index) => {
                  const direction = getTradeDirection(trade.price, index);
                  const isBuy = direction === "buy";
                  const isSell = direction === "sell";

                  return (
                    <div
                      key={trade.id}
                      className="flex justify-between text-sm hover:bg-muted/50 px-2 py-1 transition-colors"
                    >
                      <span className={`font-medium ${
                        isBuy ? "text-green-500" :
                        isSell ? "text-red-500" :
                        "text-foreground"
                      }`}>
                        {formatPrice(trade.price, quoteToken.decimals)}
                      </span>
                      <span className="text-muted-foreground">{formatSize(trade.size, baseToken.decimals)}</span>
                      <span className="text-muted-foreground text-xs">{new Date(trade.timestamp).toLocaleTimeString()}</span>
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
