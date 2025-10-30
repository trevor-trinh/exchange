"use client";

import { useState, useEffect } from "react";
import { getExchangeClient } from "@/lib/api";
import type { Token } from "@exchange/sdk";

export function FaucetForm() {
  const [tokens, setTokens] = useState<Token[]>([]);
  const [userAddress, setUserAddress] = useState("");
  const [tokenTicker, setTokenTicker] = useState("");
  const [amount, setAmount] = useState("");
  const [loading, setLoading] = useState(false);
  const [loadingTokens, setLoadingTokens] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<{ newBalance: string } | null>(null);

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
      const result = await client.rest.adminFaucet({
        userAddress,
        tokenTicker: tokenTicker.toUpperCase(),
        amount,
      });
      setSuccess(result);
      // Don't reset form so user can easily faucet more
    } catch (err: any) {
      setError(err.message || "Failed to faucet tokens");
    } finally {
      setLoading(false);
    }
  };

  // Helper to convert raw amount to human-readable
  const getSelectedTokenDecimals = () => {
    const token = tokens.find((t) => t.ticker === tokenTicker.toUpperCase());
    return token?.decimals || 6;
  };

  const calculateRawAmount = (humanAmount: string) => {
    const decimals = getSelectedTokenDecimals();
    const multiplier = Math.pow(10, decimals);
    return (parseFloat(humanAmount || "0") * multiplier).toString();
  };

  return (
    <div>
      <h2 className="text-2xl font-bold mb-4">Faucet (Admin)</h2>
      <p className="text-gray-400 mb-6">Grant tokens to any user address for testing purposes</p>

      {loadingTokens ? (
        <p className="text-gray-400">Loading available tokens...</p>
      ) : tokens.length === 0 ? (
        <div className="p-4 bg-yellow-900/20 border border-yellow-500 rounded-lg text-yellow-400 mb-6">
          No tokens available. Please create tokens first.
        </div>
      ) : (
        <form onSubmit={handleSubmit} className="space-y-4 max-w-2xl">
          <div>
            <label className="block text-sm font-medium mb-2">User Address *</label>
            <input
              type="text"
              value={userAddress}
              onChange={(e) => setUserAddress(e.target.value)}
              className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
              placeholder="user123"
              required
            />
            <p className="text-xs text-gray-500 mt-1">The address/identifier of the user to receive tokens</p>
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">Token *</label>
            <select
              value={tokenTicker}
              onChange={(e) => setTokenTicker(e.target.value)}
              className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
              required
            >
              <option value="">Select token</option>
              {tokens.map((token) => (
                <option key={token.ticker} value={token.ticker}>
                  {token.ticker} - {token.name} ({token.decimals} decimals)
                </option>
              ))}
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium mb-2">Amount (raw value) *</label>
            <input
              type="text"
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              className="w-full px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
              placeholder="1000000000"
              required
            />
            <p className="text-xs text-gray-500 mt-1">Raw amount (e.g., 1000000000 = 1000 with 6 decimals)</p>
            {tokenTicker && amount && (
              <p className="text-xs text-blue-400 mt-1">
                Human-readable:{" "}
                {(parseFloat(amount) / Math.pow(10, getSelectedTokenDecimals())).toFixed(getSelectedTokenDecimals())}{" "}
                {tokenTicker}
              </p>
            )}
          </div>

          {/* Quick amount buttons */}
          {tokenTicker && (
            <div>
              <label className="block text-sm font-medium mb-2">Quick Amounts</label>
              <div className="flex gap-2 flex-wrap">
                {[100, 1000, 10000, 100000].map((humanAmount) => (
                  <button
                    key={humanAmount}
                    type="button"
                    onClick={() => setAmount(calculateRawAmount(humanAmount.toString()))}
                    className="px-4 py-2 bg-gray-700 hover:bg-gray-600 rounded text-sm transition-colors"
                  >
                    {humanAmount} {tokenTicker}
                  </button>
                ))}
              </div>
            </div>
          )}

          {error && <div className="p-4 bg-red-900/20 border border-red-500 rounded-lg text-red-400">{error}</div>}

          {success && (
            <div className="p-4 bg-green-900/20 border border-green-500 rounded-lg">
              <p className="text-green-400 font-medium">Tokens granted successfully!</p>
              <div className="mt-2 text-sm text-gray-300">
                <p>New Balance (raw): {success.newBalance}</p>
                {tokenTicker && (
                  <p>
                    New Balance (human):{" "}
                    {(parseFloat(success.newBalance) / Math.pow(10, getSelectedTokenDecimals())).toFixed(
                      getSelectedTokenDecimals(),
                    )}{" "}
                    {tokenTicker}
                  </p>
                )}
              </div>
            </div>
          )}

          <button
            type="submit"
            disabled={loading}
            className="px-6 py-3 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed rounded-lg font-medium transition-colors"
          >
            {loading ? "Processing..." : "Grant Tokens"}
          </button>
        </form>
      )}
    </div>
  );
}
