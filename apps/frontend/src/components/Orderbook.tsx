"use client";

import { useState } from "react";
import { useOrderbook, useTrades } from "@/lib/hooks";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { formatPrice, formatSize } from "@/lib/format";

type TabType = "orderbook" | "trades";

export function Orderbook() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const { bids, asks } = useOrderbook(selectedMarketId);
  const trades = useTrades(selectedMarketId);
  const [activeTab, setActiveTab] = useState<TabType>("orderbook");

  if (!selectedMarketId || !selectedMarket) {
    return (
      <div className="bg-gray-900 rounded-lg p-6 border border-gray-800 h-full">
        <h3 className="text-lg font-bold mb-2">Orderbook</h3>
        <p className="text-gray-500 text-sm">Select a market to view orderbook</p>
      </div>
    );
  }

  // Look up token decimals
  const baseToken = tokens.find((t) => t.ticker === selectedMarket.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket.quote_ticker);

  if (!baseToken || !quoteToken) {
    return (
      <div className="bg-gray-900 rounded-lg p-6 border border-gray-800 h-full">
        <h3 className="text-lg font-bold mb-2">Orderbook</h3>
        <p className="text-gray-500 text-sm">Loading token information...</p>
      </div>
    );
  }

  return (
    <div className="bg-[#0f0f0f] rounded-xl border border-gray-800/30 backdrop-blur-sm flex flex-col h-full overflow-hidden">
      {/* Tabs */}
      <div className="flex border-b border-gray-800/30">
        <button
          onClick={() => setActiveTab("orderbook")}
          className={`flex-1 px-4 py-3 text-xs font-semibold uppercase tracking-wider transition-all ${
            activeTab === "orderbook"
              ? "text-white bg-gradient-to-b from-blue-500/10 to-transparent border-b-2 border-blue-500"
              : "text-gray-500 hover:text-gray-300 hover:bg-white/5"
          }`}
        >
          Orderbook
        </button>
        <button
          onClick={() => setActiveTab("trades")}
          className={`flex-1 px-4 py-3 text-xs font-semibold uppercase tracking-wider transition-all ${
            activeTab === "trades"
              ? "text-white bg-gradient-to-b from-blue-500/10 to-transparent border-b-2 border-blue-500"
              : "text-gray-500 hover:text-gray-300 hover:bg-white/5"
          }`}
        >
          Trades
        </button>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-hidden flex flex-col p-4">
        {activeTab === "orderbook" ? (
          <>
            <div className="flex justify-between font-semibold mb-2 text-[10px] text-gray-500 px-2 uppercase tracking-wider">
              <span>Price ({quoteToken.ticker})</span>
              <span>Size ({baseToken.ticker})</span>
            </div>

            {/* Entire orderbook scrollable container - fixed height */}
            <div className="h-[500px] overflow-y-auto scrollbar-thin scrollbar-thumb-gray-800 scrollbar-track-transparent">
              {/* Asks (Sell orders - Red) - Reversed so lowest ask is at bottom */}
              <div className="flex flex-col-reverse px-1 space-y-reverse space-y-[1px]">
                {asks.slice(0, 10).map((ask, i) => (
                  <div key={i} className="flex justify-between text-xs hover:bg-red-500/5 rounded-md px-2 py-1.5 transition-all group cursor-pointer">
                    <span className="text-red-400 font-mono font-semibold tabular-nums">{formatPrice(ask.price, quoteToken.decimals)}</span>
                    <span className="text-gray-400 font-mono tabular-nums group-hover:text-gray-300">{formatSize(ask.size, baseToken.decimals)}</span>
                  </div>
                ))}
              </div>

              {/* Spread separator - Centered */}
              <div className="flex items-center justify-center my-2 px-2">
                <div className="flex-1 border-t border-gray-800/50"></div>
                <span className="px-3 text-[10px] text-gray-600 font-bold uppercase tracking-wider">Spread</span>
                <div className="flex-1 border-t border-gray-800/50"></div>
              </div>

              {/* Bids (Buy orders - Green) */}
              <div className="px-1 space-y-[1px]">
                {bids.slice(0, 10).map((bid, i) => (
                  <div key={i} className="flex justify-between text-xs hover:bg-green-500/5 rounded-md px-2 py-1.5 transition-all group cursor-pointer">
                    <span className="text-green-400 font-mono font-semibold tabular-nums">{formatPrice(bid.price, quoteToken.decimals)}</span>
                    <span className="text-gray-400 font-mono tabular-nums group-hover:text-gray-300">{formatSize(bid.size, baseToken.decimals)}</span>
                  </div>
                ))}
              </div>
            </div>
          </>
        ) : (
          <>
            <div className="flex justify-between font-semibold mb-2 text-[10px] text-gray-500 px-2 uppercase tracking-wider">
              <span>Price</span>
              <span>Size</span>
              <span>Time</span>
            </div>

            <div className="h-[500px] overflow-y-auto scrollbar-thin scrollbar-thumb-gray-800 scrollbar-track-transparent">
              {trades.length === 0 ? (
                <p className="text-gray-500 text-xs px-2">No recent trades</p>
              ) : (
                <div className="px-1 space-y-[1px]">
                  {trades.slice(0, 50).map((trade) => (
                    <div key={trade.id} className="flex justify-between text-xs hover:bg-white/5 rounded-md px-2 py-1.5 transition-all">
                      <span className="text-white font-mono font-semibold tabular-nums">{formatPrice(trade.price, quoteToken.decimals)}</span>
                      <span className="text-gray-400 font-mono tabular-nums">{formatSize(trade.size, baseToken.decimals)}</span>
                      <span className="text-gray-600 text-[10px]">{new Date(trade.timestamp).toLocaleTimeString()}</span>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}
