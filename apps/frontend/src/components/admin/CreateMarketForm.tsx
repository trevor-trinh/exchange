"use client";

import { useState, useEffect } from "react";
import { getExchangeClient } from "@/lib/api";
import type { Market, Token } from "@exchange/sdk";

export function CreateMarketForm() {
  const [tokens, setTokens] = useState<Token[]>([]);
  const [baseTicker, setBaseTicker] = useState("");
  const [quoteTicker, setQuoteTicker] = useState("");
  const [tickSize, setTickSize] = useState("1000000"); // 0.01 with 6 decimals
  const [lotSize, setLotSize] = useState("1000000"); // 0.01 with 6 decimals
  const [minSize, setMinSize] = useState("10000000"); // 0.1 with 6 decimals
  const [makerFeeBps, setMakerFeeBps] = useState("10"); // 0.1%
  const [takerFeeBps, setTakerFeeBps] = useState("20"); // 0.2%
  const [loading, setLoading] = useState(false);
  const [loadingTokens, setLoadingTokens] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<Market | null>(null);

  useEffect(() => {
    loadTokens();
  }, []);

  const loadTokens = async () => {
    try {
      const client = getExchangeClient();
      const tokenList = await client.rest.getTokens();
      setTokens(tokenList);
    } catch (err: any) {
      console.error("Failed to load tokens:", err);
    } finally {
      setLoadingTokens(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      const client = getExchangeClient();
      const market = await client.rest.adminCreateMarket({
        baseTicker: baseTicker.toUpperCase(),
        quoteTicker: quoteTicker.toUpperCase(),
        tickSize,
        lotSize,
        minSize,
        makerFeeBps: parseInt(makerFeeBps),
        takerFeeBps: parseInt(takerFeeBps),
      });
      setSuccess(market);
      // Reset form
      setBaseTicker("");
      setQuoteTicker("");
      setTickSize("1000000");
      setLotSize("1000000");
      setMinSize("10000000");
      setMakerFeeBps("10");
      setTakerFeeBps("20");
    } catch (err: any) {
      setError(err.message || "Failed to create market");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h2 className="text-2xl font-bold mb-4">Create New Market</h2>
      <p className="text-gray-400 mb-6">Create a new trading pair from existing tokens</p>

      {loadingTokens ? (
        <p className="text-gray-400">Loading available tokens...</p>
      ) : tokens.length === 0 ? (
        <div className="p-4 bg-yellow-900/20 border border-yellow-500 rounded-lg text-yellow-400 mb-6">
          No tokens available. Please create tokens first.
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-4 max-w-2xl">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium mb-2">Base Token *</label>
              <select
                value={baseTicker}
                onChange={(e) => setBaseTicker(e.target.value)}
                className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                required
              >
                <option value="">Select base token</option>
                {tokens.map((token) => (
                  <option key={token.ticker} value={token.ticker}>
                    {token.ticker} - {token.name}
                  </option>
                ))}
              </select>
              <p className="text-xs text-gray-500 mt-1">The asset being traded (e.g., BTC)</p>
            </div>

            <div>
              <label className="block text-sm font-medium mb-2">Quote Token *</label>
              <select
                value={quoteTicker}
                onChange={(e) => setQuoteTicker(e.target.value)}
                className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                required
              >
                <option value="">Select quote token</option>
                {tokens.map((token) => (
                  <option key={token.ticker} value={token.ticker}>
                    {token.ticker} - {token.name}
                  </option>
                ))}
              </select>
              <p className="text-xs text-gray-500 mt-1">The pricing currency (e.g., USDC)</p>
            </div>
          </div>

          <div className="border-t border-gray-700 pt-4 mt-6">
            <h3 className="text-lg font-medium mb-4">Market Parameters</h3>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium mb-2">Tick Size (raw) *</label>
                <input
                  type="text"
                  value={tickSize}
                  onChange={(e) => setTickSize(e.target.value)}
                  className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder="1000000"
                  required
                />
                <p className="text-xs text-gray-500 mt-1">Minimum price increment (1000000 = 0.01)</p>
              </div>

              <div>
                <label className="block text-sm font-medium mb-2">Lot Size (raw) *</label>
                <input
                  type="text"
                  value={lotSize}
                  onChange={(e) => setLotSize(e.target.value)}
                  className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder="1000000"
                  required
                />
                <p className="text-xs text-gray-500 mt-1">Minimum size increment (1000000 = 0.01)</p>
              </div>
            </div>

            <div className="mt-4">
              <label className="block text-sm font-medium mb-2">Minimum Order Size (raw) *</label>
              <input
                type="text"
                value={minSize}
                onChange={(e) => setMinSize(e.target.value)}
                className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                placeholder="10000000"
                required
              />
              <p className="text-xs text-gray-500 mt-1">Minimum order size (10000000 = 0.1)</p>
            </div>
          </div>

          <div className="border-t border-gray-700 pt-4 mt-6">
            <h3 className="text-lg font-medium mb-4">Fee Structure</h3>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="block text-sm font-medium mb-2">Maker Fee (bps) *</label>
                <input
                  type="number"
                  value={makerFeeBps}
                  onChange={(e) => setMakerFeeBps(e.target.value)}
                  className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder="10"
                  required
                />
                <p className="text-xs text-gray-500 mt-1">In basis points (10 = 0.1%, 100 = 1%)</p>
              </div>

              <div>
                <label className="block text-sm font-medium mb-2">Taker Fee (bps) *</label>
                <input
                  type="number"
                  value={takerFeeBps}
                  onChange={(e) => setTakerFeeBps(e.target.value)}
                  className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                  placeholder="20"
                  required
                />
                <p className="text-xs text-gray-500 mt-1">In basis points (20 = 0.2%, 100 = 1%)</p>
              </div>
            </div>
          </div>

          {error && <div className="p-4 bg-red-900/20 border border-red-500 rounded-lg text-red-400">{error}</div>}

          {success && (
            <div className="p-4 bg-green-900/20 border border-green-500 rounded-lg">
              <p className="text-green-400 font-medium">Market created successfully!</p>
              <div className="mt-2 text-sm text-gray-300">
                <p>Market ID: {success.id}</p>
                <p>
                  Pair: {success.base_ticker}/{success.quote_ticker}
                </p>
              </div>
            </div>
          )}

          <button
            type="submit"
            disabled={loading}
            className="px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed rounded-lg font-medium transition-colors"
          >
            {loading ? "Creating..." : "Create Market"}
          </button>
        </form>
      )}
    </div>
  );
}
