"use client";

import { useState, useEffect } from "react";
import { getExchangeClient } from "@/lib/api";
import type { Token, Market, Balance } from "@exchange/sdk";

export function InfoPanel() {
  const [tokens, setTokens] = useState<Token[]>([]);
  const [markets, setMarkets] = useState<Market[]>([]);
  const [userAddress, setUserAddress] = useState("");
  const [balances, setBalances] = useState<Balance[]>([]);
  const [loadingInfo, setLoadingInfo] = useState(true);
  const [loadingBalances, setLoadingBalances] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadInfo();
  }, []);

  const loadInfo = async () => {
    setLoadingInfo(true);
    setError(null);
    try {
      const client = getExchangeClient();
      const [tokenList, marketList] = await Promise.all([client.rest.getTokens(), client.rest.getMarkets()]);
      setTokens(tokenList);
      setMarkets(marketList);
    } catch (err: any) {
      setError(err.message || "Failed to load info");
    } finally {
      setLoadingInfo(false);
    }
  };

  const loadBalances = async () => {
    if (!userAddress) return;
    setLoadingBalances(true);
    setError(null);
    try {
      const client = getExchangeClient();
      const balanceList = await client.rest.getBalances(userAddress);
      setBalances(balanceList);
    } catch (err: any) {
      setError(err.message || "Failed to load balances");
    } finally {
      setLoadingBalances(false);
    }
  };

  const formatBalance = (balance: string, decimals: number) => {
    return (parseFloat(balance) / Math.pow(10, decimals)).toFixed(decimals);
  };

  return (
    <div>
      <h2 className="text-2xl font-bold mb-4">Exchange Information</h2>
      <p className="text-gray-400 mb-6">View all tokens, markets, and user balances</p>

      {loadingInfo ? (
        <p className="text-gray-400">Loading...</p>
      ) : error ? (
        <div className="p-4 bg-red-900/20 border border-red-500 rounded-lg text-red-400">{error}</div>
      ) : (
        <div className="space-y-6">
          {/* Tokens Section */}
          <div>
            <div className="flex items-center justify-between mb-4">
              <h3 className="text-xl font-semibold">Tokens ({tokens.length})</h3>
              <button onClick={loadInfo} className="text-sm text-blue-400 hover:text-blue-300">
                Refresh
              </button>
            </div>
            {tokens.length === 0 ? (
              <p className="text-gray-500">No tokens created yet</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-gray-700">
                      <th className="text-left py-2 px-4">Ticker</th>
                      <th className="text-left py-2 px-4">Name</th>
                      <th className="text-left py-2 px-4">Decimals</th>
                    </tr>
                  </thead>
                  <tbody>
                    {tokens.map((token) => (
                      <tr key={token.ticker} className="border-b border-gray-800">
                        <td className="py-2 px-4 font-mono">{token.ticker}</td>
                        <td className="py-2 px-4">{token.name}</td>
                        <td className="py-2 px-4">{token.decimals}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>

          {/* Markets Section */}
          <div>
            <h3 className="text-xl font-semibold mb-4">Markets ({markets.length})</h3>
            {markets.length === 0 ? (
              <p className="text-gray-500">No markets created yet</p>
            ) : (
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-gray-700">
                      <th className="text-left py-2 px-4">Market ID</th>
                      <th className="text-left py-2 px-4">Base/Quote</th>
                      <th className="text-left py-2 px-4">Tick Size</th>
                      <th className="text-left py-2 px-4">Lot Size</th>
                      <th className="text-left py-2 px-4">Min Size</th>
                      <th className="text-left py-2 px-4">Maker Fee</th>
                      <th className="text-left py-2 px-4">Taker Fee</th>
                    </tr>
                  </thead>
                  <tbody>
                    {markets.map((market) => (
                      <tr key={market.id} className="border-b border-gray-800">
                        <td className="py-2 px-4 font-mono">{market.id}</td>
                        <td className="py-2 px-4">
                          {market.base_ticker}/{market.quote_ticker}
                        </td>
                        <td className="py-2 px-4 font-mono text-sm">{market.tick_size}</td>
                        <td className="py-2 px-4 font-mono text-sm">{market.lot_size}</td>
                        <td className="py-2 px-4 font-mono text-sm">{market.min_size}</td>
                        <td className="py-2 px-4">{market.maker_fee_bps / 100}%</td>
                        <td className="py-2 px-4">{market.taker_fee_bps / 100}%</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>

          {/* User Balances Section */}
          <div className="border-t border-gray-700 pt-6">
            <h3 className="text-xl font-semibold mb-4">Check User Balances</h3>
            <div className="flex gap-2 mb-4">
              <input
                type="text"
                value={userAddress}
                onChange={(e) => setUserAddress(e.target.value)}
                className="flex-1 px-4 py-2 bg-gray-800 border border-gray-700 rounded-lg focus:outline-none focus:border-blue-500"
                placeholder="Enter user address"
              />
              <button
                onClick={loadBalances}
                disabled={!userAddress || loadingBalances}
                className="px-6 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-700 disabled:cursor-not-allowed rounded-lg transition-colors"
              >
                {loadingBalances ? "Loading..." : "Load Balances"}
              </button>
            </div>

            {balances.length > 0 && (
              <div className="overflow-x-auto">
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-gray-700">
                      <th className="text-left py-2 px-4">Token</th>
                      <th className="text-left py-2 px-4">Available (raw)</th>
                      <th className="text-left py-2 px-4">Available (human)</th>
                      <th className="text-left py-2 px-4">Locked (raw)</th>
                      <th className="text-left py-2 px-4">Locked (human)</th>
                    </tr>
                  </thead>
                  <tbody>
                    {balances.map((balance) => {
                      const token = tokens.find((t) => t.ticker === balance.ticker);
                      const decimals = token?.decimals || 6;
                      return (
                        <tr key={balance.ticker} className="border-b border-gray-800">
                          <td className="py-2 px-4 font-mono">{balance.ticker}</td>
                          <td className="py-2 px-4 font-mono text-sm">{balance.available}</td>
                          <td className="py-2 px-4">{formatBalance(balance.available, decimals)}</td>
                          <td className="py-2 px-4 font-mono text-sm">{balance.locked}</td>
                          <td className="py-2 px-4">{formatBalance(balance.locked, decimals)}</td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
