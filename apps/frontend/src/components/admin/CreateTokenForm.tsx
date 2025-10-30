"use client";

import { useState } from "react";
import { getExchangeClient } from "@/lib/api";
import type { Token } from "@exchange/sdk";

export function CreateTokenForm() {
  const [ticker, setTicker] = useState("");
  const [name, setName] = useState("");
  const [decimals, setDecimals] = useState("6");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<Token | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    setSuccess(null);

    try {
      const client = getExchangeClient();
      const token = await client.rest.adminCreateToken({
        ticker: ticker.toUpperCase(),
        name,
        decimals: parseInt(decimals),
      });
      setSuccess(token);
      // Reset form
      setTicker("");
      setName("");
      setDecimals("6");
    } catch (err: any) {
      setError(err.message || "Failed to create token");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h2 className="text-2xl font-bold mb-4">Create New Token</h2>
      <p className="text-gray-400 mb-6">Create a new token that can be used in trading pairs</p>

      <form onSubmit={handleSubmit} className="space-y-4 max-w-2xl">
        <div>
          <label className="block text-sm font-medium mb-2">Ticker Symbol *</label>
          <input
            type="text"
            value={ticker}
            onChange={(e) => setTicker(e.target.value)}
            className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
            placeholder="BTC"
            required
          />
          <p className="text-xs text-gray-500 mt-1">Uppercase abbreviation (e.g., BTC, ETH, USDC)</p>
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">Name *</label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
            placeholder="Bitcoin"
            required
          />
        </div>

        <div>
          <label className="block text-sm font-medium mb-2">Decimals *</label>
          <input
            type="number"
            value={decimals}
            onChange={(e) => setDecimals(e.target.value)}
            min="0"
            max="18"
            className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
            required
          />
          <p className="text-xs text-gray-500 mt-1">Number of decimal places (typically 6-18)</p>
        </div>

        {error && <div className="p-4 bg-red-900/20 border border-red-500 rounded-lg text-red-400">{error}</div>}

        {success && (
          <div className="p-4 bg-green-900/20 border border-green-500 rounded-lg">
            <p className="text-green-400 font-medium">Token created successfully!</p>
            <div className="mt-2 text-sm text-gray-300">
              <p>Ticker: {success.ticker}</p>
              <p>Name: {success.name}</p>
              <p>Decimals: {success.decimals}</p>
            </div>
          </div>
        )}

        <button
          type="submit"
          disabled={loading}
          className="px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed rounded-lg font-medium transition-colors"
        >
          {loading ? "Creating..." : "Create Token"}
        </button>
      </form>
    </div>
  );
}
