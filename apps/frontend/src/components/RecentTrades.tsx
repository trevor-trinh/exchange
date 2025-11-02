"use client";

import { useState, useEffect } from "react";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { getExchangeClient } from "@/lib/api";
import { formatPrice, formatSize, formatTime } from "@/lib/format";
import type { Trade } from "@exchange/sdk";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";

export function RecentTrades() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);

  const [trades, setTrades] = useState<Trade[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!userAddress || !isAuthenticated || !selectedMarketId) {
      setTrades([]);
      return;
    }

    const fetchTrades = async () => {
      setLoading(true);
      try {
        const client = getExchangeClient();
        const result = await client.getTrades(userAddress, selectedMarketId);
        setTrades(result);
      } catch (err) {
        console.error("Failed to fetch trades:", err);
        setTrades([]);
      } finally {
        setLoading(false);
      }
    };

    fetchTrades();
    const interval = setInterval(fetchTrades, 2000); // Refresh every 2 seconds

    return () => clearInterval(interval);
  }, [userAddress, isAuthenticated, selectedMarketId]);

  if (!selectedMarketId || !selectedMarket) {
    return <p className="text-muted-foreground text-sm">Select a market to view trades</p>;
  }

  const baseToken = tokens.find((t) => t.ticker === selectedMarket.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket.quote_ticker);

  if (!baseToken || !quoteToken) {
    return <p className="text-muted-foreground text-sm">Loading token information...</p>;
  }

  return (
    <div>
      <div className="overflow-auto max-h-80">
        {loading && !trades.length ? (
          <p className="text-muted-foreground text-sm">Loading trades...</p>
        ) : !isAuthenticated || !userAddress ? (
          <p className="text-muted-foreground text-sm">Connect your wallet to view your trades</p>
        ) : trades.length === 0 ? (
          <p className="text-muted-foreground text-sm">No trades found</p>
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
              {trades.slice(0, 50).map((trade) => {
                // Determine if this user was the buyer or seller
                const isBuyer = trade.buyer_address === userAddress;
                const side = isBuyer ? "buy" : "sell";

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
                          side === "buy"
                            ? "bg-green-500/10 text-green-500 border border-green-500/20"
                            : "bg-red-500/10 text-red-500 border border-red-500/20"
                        }`}
                      >
                        {side === "buy" ? "Buy" : "Sell"}
                      </span>
                    </TableCell>
                    <TableCell className="text-muted-foreground text-xs">
                      {formatTime(trade.timestamp)}
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
