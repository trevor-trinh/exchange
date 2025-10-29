"use client";

import { useTrades } from "@/lib/hooks";
import { useExchangeStore } from "@/lib/store";

export function TradeHistory() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const trades = useTrades(selectedMarketId);

  if (!selectedMarketId) {
    return (
      <div className="p-4 border rounded">
        <h3 className="text-lg font-bold mb-2">Recent Trades</h3>
        <p className="text-gray-500">Select a market to view trades</p>
      </div>
    );
  }

  return (
    <div className="p-4 border rounded">
      <h3 className="text-lg font-bold mb-4">Recent Trades</h3>
      <div className="overflow-auto max-h-96">
        <table className="w-full text-sm">
          <thead>
            <tr className="border-b">
              <th className="text-left p-2">Price</th>
              <th className="text-left p-2">Size</th>
              <th className="text-left p-2">Time</th>
            </tr>
          </thead>
          <tbody>
            {trades.slice(0, 50).map((trade) => (
              <tr key={trade.id} className="border-b">
                <td className="p-2">{parseFloat(trade.price).toFixed(2)}</td>
                <td className="p-2">{(parseFloat(trade.size) / 1e6).toFixed(4)}</td>
                <td className="p-2 text-gray-500">{new Date(trade.timestamp).toLocaleTimeString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
