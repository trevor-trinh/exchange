"use client";

import { useState, useEffect } from "react";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { getExchangeClient } from "@/lib/api";
import { formatPrice, formatSize, formatTime } from "@/lib/format";
import type { Order } from "@exchange/sdk";
import { Input } from "@/components/ui/input";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";

export function RecentOrders() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);

  const [orders, setOrders] = useState<Order[]>([]);
  const [loading, setLoading] = useState(false);

  const baseToken = tokens.find((t) => t.ticker === selectedMarket?.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket?.quote_ticker);

  useEffect(() => {
    if (!userAddress || !isAuthenticated || !selectedMarketId) {
      setOrders([]);
      return;
    }

    const fetchOrders = async () => {
      setLoading(true);
      try {
        const client = getExchangeClient();
        const result = await client.getOrders({
          userAddress,
          marketId: selectedMarketId,
          limit: 50,
        });
        setOrders(result);
      } catch (err) {
        console.error("Failed to fetch orders:", err);
        setOrders([]);
      } finally {
        setLoading(false);
      }
    };

    fetchOrders();
    const interval = setInterval(fetchOrders, 2000); // Refresh every 2 seconds

    return () => clearInterval(interval);
  }, [userAddress, isAuthenticated, selectedMarketId]);

  if (!selectedMarketId || !selectedMarket || !baseToken || !quoteToken) {
    return <p className="text-muted-foreground text-sm">Select a market to view orders</p>;
  }

  return (
    <div>
      <div className="overflow-auto max-h-80">
        {loading && !orders.length ? (
          <p className="text-muted-foreground text-sm">Loading orders...</p>
        ) : !isAuthenticated || !userAddress ? (
          <p className="text-muted-foreground text-sm">Connect your wallet to view orders</p>
        ) : orders.length === 0 ? (
          <p className="text-muted-foreground text-sm">No orders found</p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Side</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Price</TableHead>
                <TableHead>Size</TableHead>
                <TableHead>Filled</TableHead>
                <TableHead>Status</TableHead>
                <TableHead>Time</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {orders.map((order) => (
                <TableRow key={order.id}>
                  <TableCell>
                    <span className={`font-bold ${order.side === "buy" ? "text-green-500" : "text-red-500"}`}>
                      {order.side === "buy" ? "Buy" : "Sell"}
                    </span>
                  </TableCell>
                  <TableCell className="text-muted-foreground">
                    {order.order_type === "limit" ? "Limit" : "Market"}
                  </TableCell>
                  <TableCell className="font-mono">{formatPrice(order.price, quoteToken.decimals)}</TableCell>
                  <TableCell className="font-mono text-muted-foreground">
                    {formatSize(order.size, baseToken.decimals)}
                  </TableCell>
                  <TableCell className="font-mono text-muted-foreground">
                    {formatSize(order.filled_size, baseToken.decimals)}
                  </TableCell>
                  <TableCell>
                    <span
                      className={`text-xs px-2 py-1 font-semibold uppercase tracking-wide ${
                        order.status === "filled"
                          ? "bg-green-500/10 text-green-500 border border-green-500/20"
                          : order.status === "open"
                            ? "bg-blue-500/10 text-blue-500 border border-blue-500/20"
                            : order.status === "partially_filled"
                              ? "bg-yellow-500/10 text-yellow-500 border border-yellow-500/20"
                              : "bg-muted text-muted-foreground border border-border"
                      }`}
                    >
                      {order.status.replace("_", " ")}
                    </span>
                  </TableCell>
                  <TableCell className="text-muted-foreground text-xs">
                    {formatTime(order.created_at)}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </div>
    </div>
  );
}
