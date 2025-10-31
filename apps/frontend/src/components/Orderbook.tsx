"use client";

import { useOrderbook } from "@/lib/hooks";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { formatPrice, formatSize } from "@/lib/format";

export function Orderbook() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const { bids, asks } = useOrderbook(selectedMarketId);

  if (!selectedMarketId || !selectedMarket) {
    return (
      <div className="p-4 border rounded">
        <h3 className="text-lg font-bold mb-2">Orderbook</h3>
        <p className="text-gray-500">Select a market to view orderbook</p>
      </div>
    );
  }

  // Look up token decimals
  const baseToken = tokens.find((t) => t.ticker === selectedMarket.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket.quote_ticker);

  if (!baseToken || !quoteToken) {
    return (
      <div className="p-4 border rounded">
        <h3 className="text-lg font-bold mb-2">Orderbook</h3>
        <p className="text-gray-500">Loading token information...</p>
      </div>
    );
  }

  return (
    <div className="p-4 border rounded">
      <h3 className="text-lg font-bold mb-4">Orderbook</h3>

      <div className="grid grid-cols-2 gap-4">
        {/* Asks (Sell orders - Red) */}
        <div>
          <div className="flex justify-between font-bold mb-2 text-sm text-gray-500">
            <span>Price ({quoteToken.ticker})</span>
            <span>Size ({baseToken.ticker})</span>
          </div>
          <div className="space-y-1">
            {asks
              .slice(0, 15)
              .reverse()
              .map((ask, i) => (
                <div key={i} className="flex justify-between text-sm text-red-500">
                  <span>{formatPrice(ask.price, quoteToken.decimals)}</span>
                  <span>{formatSize(ask.size, baseToken.decimals)}</span>
                </div>
              ))}
          </div>
        </div>

        {/* Bids (Buy orders - Green) */}
        <div>
          <div className="flex justify-between font-bold mb-2 text-sm text-gray-500">
            <span>Price ({quoteToken.ticker})</span>
            <span>Size ({baseToken.ticker})</span>
          </div>
          <div className="space-y-1">
            {bids.slice(0, 15).map((bid, i) => (
              <div key={i} className="flex justify-between text-sm text-green-500">
                <span>{formatPrice(bid.price, quoteToken.decimals)}</span>
                <span>{formatSize(bid.size, baseToken.decimals)}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
