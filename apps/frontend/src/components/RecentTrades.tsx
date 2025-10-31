"use client";

import { useTrades } from "@/lib/hooks";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { formatPrice, formatSize } from "@/lib/format";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export function RecentTrades() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const trades = useTrades(selectedMarketId);

  if (!selectedMarketId || !selectedMarket) {
    return <p className="text-muted-foreground text-sm">Select a market to view trades</p>;
  }

  const baseToken = tokens.find((t) => t.ticker === selectedMarket.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket.quote_ticker);

  if (!baseToken || !quoteToken) {
    return <p className="text-muted-foreground text-sm">Loading token information...</p>;
  }

  // Determine if trade is buy or sell based on price movement
  const getTradeDirection = (price: string, index: number) => {
    if (index >= trades.length - 1) return "neutral";
    const prevPrice = trades[index + 1].price;
    return parseFloat(price) >= parseFloat(prevPrice) ? "buy" : "sell";
  };

  return (
    <div>
      <div className="overflow-auto max-h-80">
        {trades.length === 0 ? (
          <p className="text-muted-foreground text-sm">No recent trades</p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Price ({quoteToken.ticker})</TableHead>
                <TableHead>Size ({baseToken.ticker})</TableHead>
                <TableHead>Side</TableHead>
                <TableHead>Time</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {trades.slice(0, 50).map((trade, index) => {
                const direction = getTradeDirection(trade.price, index);
                return (
                  <TableRow key={trade.id}>
                    <TableCell className="font-mono font-semibold">
                      {formatPrice(trade.price, quoteToken.decimals)}
                    </TableCell>
                    <TableCell className="font-mono text-muted-foreground">
                      {formatSize(trade.size, baseToken.decimals)}
                    </TableCell>
                    <TableCell>
                      <span
                        className={`text-xs px-2 py-1 font-semibold uppercase tracking-wide ${
                          direction === "buy"
                            ? "bg-green-500/10 text-green-500 border border-green-500/20"
                            : direction === "sell"
                            ? "bg-red-500/10 text-red-500 border border-red-500/20"
                            : "bg-muted text-muted-foreground border border-border"
                        }`}
                      >
                        {direction === "buy" ? "Buy" : direction === "sell" ? "Sell" : "—"}
                      </span>
                    </TableCell>
                    <TableCell className="text-muted-foreground text-xs">
                      {new Date(trade.timestamp).toLocaleTimeString()}
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        )}
      </div>
    </div>
  );
}
