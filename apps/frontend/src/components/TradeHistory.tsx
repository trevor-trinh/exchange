"use client";

import { useTrades } from "@/lib/hooks";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { formatPrice, formatSize } from "@/lib/format";

export function TradeHistory() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const trades = useTrades(selectedMarketId);

  if (!selectedMarketId || !selectedMarket) {
    return (
      <div className="p-4 border rounded">
        <h3 className="text-lg font-bold mb-2">Recent Trades</h3>
        <p className="text-gray-500">Select a market to view trades</p>
      </div>
    );
  }

  // Look up token decimals
  const baseToken = tokens.find((t) => t.ticker === selectedMarket.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket.quote_ticker);

  if (!baseToken || !quoteToken) {
    return (
      <div className="p-4 border rounded">
        <h3 className="text-lg font-bold mb-2">Recent Trades</h3>
        <p className="text-gray-500">Loading token information...</p>
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
              <th className="text-left p-2">Price ({quoteToken.ticker})</th>
              <th className="text-left p-2">Size ({baseToken.ticker})</th>
              <th className="text-left p-2">Time</th>
            </tr>
          </thead>
          <tbody>
            {trades.slice(0, 50).map((trade) => (
              <tr key={trade.id} className="border-b">
                <td className="p-2">{formatPrice(trade.price, quoteToken.decimals)}</td>
                <td className="p-2">{formatSize(trade.size, baseToken.decimals)}</td>
                <td className="p-2 text-gray-500">{new Date(trade.timestamp).toLocaleTimeString()}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
