"use client";

import { useOrderbook } from "@/lib/hooks";
import { useExchangeStore } from "@/lib/store";

export function Orderbook() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const { bids, asks } = useOrderbook(selectedMarketId);

  if (!selectedMarketId) {
    return (
      <div className="p-4 border rounded">
        <h3 className="text-lg font-bold mb-2">Orderbook</h3>
        <p className="text-gray-500">Select a market to view orderbook</p>
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
            <span>Price (USDC)</span>
            <span>Size (BTC)</span>
          </div>
          <div className="space-y-1">
            {asks
              .slice(0, 15)
              .reverse()
              .map((ask, i) => (
                <div key={i} className="flex justify-between text-sm text-red-500">
                  <span>{parseFloat(ask.price).toFixed(2)}</span>
                  <span>{(parseFloat(ask.size) / 1e6).toFixed(4)}</span>
                </div>
              ))}
          </div>
        </div>

        {/* Bids (Buy orders - Green) */}
        <div>
          <div className="flex justify-between font-bold mb-2 text-sm text-gray-500">
            <span>Price (USDC)</span>
            <span>Size (BTC)</span>
          </div>
          <div className="space-y-1">
            {bids.slice(0, 15).map((bid, i) => (
              <div key={i} className="flex justify-between text-sm text-green-500">
                <span>{parseFloat(bid.price).toFixed(2)}</span>
                <span>{(parseFloat(bid.size) / 1e6).toFixed(4)}</span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
