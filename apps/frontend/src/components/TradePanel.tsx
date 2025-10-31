"use client";

import { useState } from "react";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { getExchangeClient } from "@/lib/api";

export function TradePanel() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);

  const [side, setSide] = useState<"buy" | "sell">("buy");
  const [orderType, setOrderType] = useState<"limit" | "market">("limit");
  const [price, setPrice] = useState("");
  const [size, setSize] = useState("");
  const [userAddress, setUserAddress] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  if (!selectedMarketId || !selectedMarket) {
    return (
      <div className="bg-[#0f0f0f] rounded-xl p-6 border border-gray-800/30">
        <h3 className="text-sm font-semibold mb-4 uppercase tracking-wider">Trade</h3>
        <p className="text-gray-500 text-xs">Select a market to trade</p>
      </div>
    );
  }

  const baseToken = tokens.find((t) => t.ticker === selectedMarket.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket.quote_ticker);

  if (!baseToken || !quoteToken) {
    return (
      <div className="bg-[#0f0f0f] rounded-xl p-6 border border-gray-800/30">
        <h3 className="text-sm font-semibold mb-4 uppercase tracking-wider">Trade</h3>
        <p className="text-gray-500 text-xs">Loading token information...</p>
      </div>
    );
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(null);
    setLoading(true);

    try {
      if (!userAddress.trim()) {
        throw new Error("User address is required");
      }
      if (!price.trim() && orderType === "limit") {
        throw new Error("Price is required for limit orders");
      }
      if (!size.trim()) {
        throw new Error("Size is required");
      }

      const client = getExchangeClient();

      // For demo purposes, using a simple signature
      // In production, this would be a proper cryptographic signature
      const signature = `${userAddress}:${Date.now()}`;

      const result = await client.placeOrder({
        userAddress: userAddress.trim(),
        marketId: selectedMarketId,
        side: side === "buy" ? "buy" : "sell",
        orderType: orderType === "limit" ? "limit" : "market",
        price: orderType === "limit" ? price : "0",
        size,
        signature,
      });

      setSuccess(`Order placed successfully! Order ID: ${result.order.id.slice(0, 8)}...`);
      setPrice("");
      setSize("");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to place order");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bg-[#0f0f0f] rounded-xl p-5 border border-gray-800/30 h-full">
      {/* Buy/Sell Tabs */}
      <div className="grid grid-cols-2 gap-1.5 mb-5 bg-[#141414] p-1 rounded-lg">
        <button
          onClick={() => setSide("buy")}
          className={`py-2.5 px-4 rounded-md text-xs font-bold uppercase tracking-wide transition-all ${
            side === "buy"
              ? "bg-gradient-to-br from-green-500 to-green-600 text-white shadow-lg shadow-green-500/20"
              : "text-gray-500 hover:text-gray-300"
          }`}
        >
          Buy
        </button>
        <button
          onClick={() => setSide("sell")}
          className={`py-2.5 px-4 rounded-md text-xs font-bold uppercase tracking-wide transition-all ${
            side === "sell"
              ? "bg-gradient-to-br from-red-500 to-red-600 text-white shadow-lg shadow-red-500/20"
              : "text-gray-500 hover:text-gray-300"
          }`}
        >
          Sell
        </button>
      </div>

      {/* Order Type */}
      <div className="mb-4">
        <label className="block text-[10px] font-semibold text-gray-500 mb-2 uppercase tracking-wider">Order Type</label>
        <div className="grid grid-cols-2 gap-1.5 bg-[#141414] p-1 rounded-lg">
          <button
            onClick={() => setOrderType("limit")}
            className={`py-2 px-3 rounded-md text-xs font-semibold transition-all ${
              orderType === "limit"
                ? "bg-blue-500 text-white"
                : "text-gray-500 hover:text-gray-300"
            }`}
          >
            Limit
          </button>
          <button
            onClick={() => setOrderType("market")}
            className={`py-2 px-3 rounded-md text-xs font-semibold transition-all ${
              orderType === "market"
                ? "bg-blue-500 text-white"
                : "text-gray-500 hover:text-gray-300"
            }`}
          >
            Market
          </button>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="space-y-3.5">
        {/* User Address */}
        <div>
          <label className="block text-[10px] font-semibold text-gray-500 mb-1.5 uppercase tracking-wider">User Address</label>
          <input
            type="text"
            value={userAddress}
            onChange={(e) => setUserAddress(e.target.value)}
            placeholder="Enter your address"
            className="w-full bg-[#141414] border border-gray-800/50 rounded-lg px-3 py-2.5 text-white text-xs placeholder-gray-600 focus:outline-none focus:border-blue-500/50 hover:border-gray-700 transition-colors"
          />
        </div>

        {/* Price - Only for limit orders */}
        {orderType === "limit" && (
          <div>
            <label className="block text-[10px] font-semibold text-gray-500 mb-1.5 uppercase tracking-wider">
              Price ({quoteToken.ticker})
            </label>
            <input
              type="text"
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              placeholder="0.00"
              className="w-full bg-[#141414] border border-gray-800/50 rounded-lg px-3 py-2.5 text-white text-xs font-mono placeholder-gray-600 focus:outline-none focus:border-blue-500/50 hover:border-gray-700 transition-colors"
            />
          </div>
        )}

        {/* Size */}
        <div>
          <label className="block text-[10px] font-semibold text-gray-500 mb-1.5 uppercase tracking-wider">
            Size ({baseToken.ticker})
          </label>
          <input
            type="text"
            value={size}
            onChange={(e) => setSize(e.target.value)}
            placeholder="0.00"
            className="w-full bg-[#141414] border border-gray-800/50 rounded-lg px-3 py-2.5 text-white text-xs font-mono placeholder-gray-600 focus:outline-none focus:border-blue-500/50 hover:border-gray-700 transition-colors"
          />
        </div>

        {/* Total - Only for limit orders */}
        {orderType === "limit" && price && size && (
          <div className="bg-[#141414] border border-gray-800/30 rounded-lg p-3">
            <div className="flex justify-between items-center text-xs">
              <span className="text-gray-500 font-semibold uppercase tracking-wider">Total</span>
              <span className="text-white font-mono font-semibold">
                {(parseFloat(price) * parseFloat(size)).toFixed(quoteToken.decimals)}{" "}
                {quoteToken.ticker}
              </span>
            </div>
          </div>
        )}

        {/* Error/Success Messages */}
        {error && (
          <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-2.5 text-red-400 text-xs">
            {error}
          </div>
        )}
        {success && (
          <div className="bg-green-500/10 border border-green-500/30 rounded-lg p-2.5 text-green-400 text-xs">
            {success}
          </div>
        )}

        {/* Submit Button */}
        <button
          type="submit"
          disabled={loading}
          className={`w-full py-3 px-4 rounded-lg text-xs font-bold uppercase tracking-wide transition-all ${
            loading
              ? "bg-gray-800 text-gray-600 cursor-not-allowed"
              : side === "buy"
              ? "bg-gradient-to-br from-green-500 to-green-600 hover:from-green-600 hover:to-green-700 text-white shadow-lg shadow-green-500/20 hover:shadow-green-500/30"
              : "bg-gradient-to-br from-red-500 to-red-600 hover:from-red-600 hover:to-red-700 text-white shadow-lg shadow-red-500/20 hover:shadow-red-500/30"
          }`}
        >
          {loading ? "Placing Order..." : `${side === "buy" ? "Buy" : "Sell"} ${baseToken.ticker}`}
        </button>
      </form>
    </div>
  );
}
