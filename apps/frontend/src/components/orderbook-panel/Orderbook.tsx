"use client";

import { useOrderbook, useTrades } from "@/lib/hooks";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { Card, CardContent } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { OrderbookRow } from "./OrderbookRow";
import { TradeRow } from "./TradeRow";
import { SpreadIndicator } from "./SpreadIndicator";

export function Orderbook() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const setSelectedPrice = useExchangeStore((state) => state.setSelectedPrice);

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
    <Card className="flex flex-col h-full gap-0 py-0 overflow-hidden min-w-0">
      <Tabs defaultValue="orderbook" className="flex-1 flex flex-col gap-0 min-h-0 min-w-0">
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
          <OrderbookHeader
            columns={[`Price (${selectedMarket.quote_ticker})`, `Size (${selectedMarket.base_ticker})`]}
          />

          <div className="flex-1 flex flex-col min-h-0 overflow-hidden">
            {/* Asks (Sell orders - Red) - Takes up top half, aligned to bottom */}
            <div className="flex-1 flex flex-col justify-end overflow-hidden">
              <div className="flex flex-col-reverse">
                {asksWithCumulative.map((ask, i) => (
                  <OrderbookRow
                    key={i}
                    price={ask.priceDisplay}
                    priceValue={ask.priceValue}
                    size={ask.sizeDisplay}
                    cumulative={ask.cumulative}
                    maxCumulative={maxAskCumulative}
                    type="ask"
                    onClick={setSelectedPrice}
                  />
                ))}
              </div>
            </div>

            {/* Spread separator - Always in the middle */}
            <SpreadIndicator spreadPercentage={spread.spreadPercentage} />

            {/* Bids (Buy orders - Green) - Takes up bottom half, aligned to top */}
            <div className="flex-1 flex flex-col justify-start overflow-hidden">
              <div>
                {bidsWithCumulative.map((bid, i) => (
                  <OrderbookRow
                    key={i}
                    price={bid.priceDisplay}
                    priceValue={bid.priceValue}
                    size={bid.sizeDisplay}
                    cumulative={bid.cumulative}
                    maxCumulative={maxBidCumulative}
                    type="bid"
                    onClick={setSelectedPrice}
                  />
                ))}
              </div>
            </div>
          </div>
        </TabsContent>
        <TabsContent
          value="trades"
          className="flex-1 overflow-hidden flex flex-col mt-0 min-h-0 data-[state=active]:animate-in data-[state=active]:fade-in-0 data-[state=active]:slide-in-from-bottom-1 data-[state=active]:duration-200"
        >
          <OrderbookHeader columns={["Price", "Size", "Time"]} />

          <div className="flex-1 overflow-y-auto min-h-0">
            {trades.length === 0 ? (
              <div className="flex items-center justify-center h-32">
                <p className="text-muted-foreground text-xs">No recent trades</p>
              </div>
            ) : (
              <div>
                {trades.slice(0, 50).map((trade) => {
                  const timeStr =
                    trade.timestamp instanceof Date
                      ? trade.timestamp.toLocaleTimeString([], {
                          hour: "2-digit",
                          minute: "2-digit",
                          second: "2-digit",
                        })
                      : new Date(trade.timestamp).toLocaleTimeString([], {
                          hour: "2-digit",
                          minute: "2-digit",
                          second: "2-digit",
                        });

                  return (
                    <TradeRow
                      key={trade.id}
                      price={trade.priceDisplay}
                      size={trade.sizeDisplay}
                      time={timeStr}
                      side={trade.side as "buy" | "sell"}
                    />
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

function OrderbookHeader({ columns }: { columns: string[] }) {
  const isThree = columns.length === 3;
  return (
    <div
      className={
        `${isThree ? "grid grid-cols-[1fr_0.8fr_1.2fr]" : "flex justify-between"} ` +
        "font-medium text-[10px] text-muted-foreground/70 px-3 pt-1.5 pb-1 uppercase tracking-wider shrink-0 border-b border-border/50 bg-muted/20"
      }
    >
      {columns.map((col, i) => {
        const isLast = i === columns.length - 1;
        const isSecond = i === 1;
        const rightAlign = columns.length === 3 ? isSecond || isLast : isLast;
        return (
          <span key={i} className={`${rightAlign ? "text-right" : ""} whitespace-nowrap truncate`}>
            {col}
          </span>
        );
      })}
    </div>
  );
}
