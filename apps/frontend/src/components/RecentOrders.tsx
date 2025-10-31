"use client";

import { useState, useEffect } from "react";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { getExchangeClient } from "@/lib/api";
import { formatPrice, formatSize } from "@/lib/format";
import type { Order } from "@exchange/sdk";

export function RecentOrders() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);

  const [orders, setOrders] = useState<Order[]>([]);
  const [userAddress, setUserAddress] = useState("");
  const [loading, setLoading] = useState(false);

  const baseToken = tokens.find((t) => t.ticker === selectedMarket?.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket?.quote_ticker);

  useEffect(() => {
    if (!userAddress.trim() || !selectedMarketId) {
      setOrders([]);
      return;
    }

    const fetchOrders = async () => {
      setLoading(true);
      try {
        const client = getExchangeClient();
        const result = await client.getOrders({
          userAddress: userAddress.trim(),
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
  }, [userAddress, selectedMarketId]);

  if (!selectedMarketId || !selectedMarket || !baseToken || !quoteToken) {
    return <p className="text-gray-500 text-sm">Select a market to view orders</p>;
  }

  return (
    <div>
      <div className="mb-4">
        <input
          type="text"
          value={userAddress}
          onChange={(e) => setUserAddress(e.target.value)}
          placeholder="Enter your address to view orders"
          className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 text-sm"
        />
      </div>

      <div className="overflow-auto max-h-80">
        {loading && !orders.length ? (
          <p className="text-gray-500 text-sm">Loading orders...</p>
        ) : !userAddress.trim() ? (
          <p className="text-gray-500 text-sm">Enter your address to view orders</p>
        ) : orders.length === 0 ? (
          <p className="text-gray-500 text-sm">No orders found</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-800">
                <th className="text-left p-2 text-gray-400 font-medium">Side</th>
                <th className="text-left p-2 text-gray-400 font-medium">Type</th>
                <th className="text-left p-2 text-gray-400 font-medium">Price</th>
                <th className="text-left p-2 text-gray-400 font-medium">Size</th>
                <th className="text-left p-2 text-gray-400 font-medium">Filled</th>
                <th className="text-left p-2 text-gray-400 font-medium">Status</th>
                <th className="text-left p-2 text-gray-400 font-medium">Time</th>
              </tr>
            </thead>
            <tbody>
              {orders.map((order) => (
                <tr key={order.id} className="border-b border-gray-800 hover:bg-gray-800/50">
                  <td className="p-2">
                    <span
                      className={`font-medium ${
                        order.side === "buy" ? "text-green-500" : "text-red-500"
                      }`}
                    >
                      {order.side === "buy" ? "Buy" : "Sell"}
                    </span>
                  </td>
                  <td className="p-2 text-gray-300">
                    {order.order_type === "limit" ? "Limit" : "Market"}
                  </td>
                  <td className="p-2 text-gray-300">
                    {formatPrice(order.price, quoteToken.decimals)}
                  </td>
                  <td className="p-2 text-gray-300">
                    {formatSize(order.size, baseToken.decimals)}
                  </td>
                  <td className="p-2 text-gray-300">
                    {formatSize(order.filled_size, baseToken.decimals)}
                  </td>
                  <td className="p-2">
                    <span
                      className={`text-xs px-2 py-1 rounded ${
                        order.status === "filled"
                          ? "bg-green-900/30 text-green-400"
                          : order.status === "open"
                          ? "bg-blue-900/30 text-blue-400"
                          : order.status === "partially_filled"
                          ? "bg-yellow-900/30 text-yellow-400"
                          : "bg-gray-800 text-gray-400"
                      }`}
                    >
                      {order.status.replace("_", " ")}
                    </span>
                  </td>
                  <td className="p-2 text-gray-500">
                    {new Date(order.created_at).toLocaleTimeString()}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
