"use client";

import { useState, useEffect } from "react";
import { useExchangeStore } from "@/lib/store";
import { getExchangeClient } from "@/lib/api";
import { formatSize } from "@/lib/format";
import type { Balance } from "@exchange/sdk";

export function Balances() {
  const tokens = useExchangeStore((state) => state.tokens);
  const [balances, setBalances] = useState<Balance[]>([]);
  const [userAddress, setUserAddress] = useState("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!userAddress.trim()) {
      setBalances([]);
      return;
    }

    const fetchBalances = async () => {
      setLoading(true);
      try {
        const client = getExchangeClient();
        const result = await client.getBalances(userAddress.trim());
        setBalances(result);
      } catch (err) {
        console.error("Failed to fetch balances:", err);
        setBalances([]);
      } finally {
        setLoading(false);
      }
    };

    fetchBalances();
    const interval = setInterval(fetchBalances, 3000); // Refresh every 3 seconds

    return () => clearInterval(interval);
  }, [userAddress]);

  return (
    <div>
      <div className="mb-4">
        <input
          type="text"
          value={userAddress}
          onChange={(e) => setUserAddress(e.target.value)}
          placeholder="Enter your address to view balances"
          className="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 text-sm"
        />
      </div>

      <div className="overflow-auto max-h-80">
        {loading && !balances.length ? (
          <p className="text-gray-500 text-sm">Loading balances...</p>
        ) : !userAddress.trim() ? (
          <p className="text-gray-500 text-sm">Enter your address to view balances</p>
        ) : balances.length === 0 ? (
          <p className="text-gray-500 text-sm">No balances found</p>
        ) : (
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-800">
                <th className="text-left p-2 text-gray-400 font-medium">Token</th>
                <th className="text-right p-2 text-gray-400 font-medium">Available</th>
                <th className="text-right p-2 text-gray-400 font-medium">In Orders</th>
                <th className="text-right p-2 text-gray-400 font-medium">Total</th>
              </tr>
            </thead>
            <tbody>
              {balances.map((balance) => {
                const token = tokens.find((t) => t.ticker === balance.token_ticker);
                if (!token) return null;

                const available = BigInt(balance.amount) - BigInt(balance.open_interest);
                const total = BigInt(balance.amount);

                return (
                  <tr key={balance.token_ticker} className="border-b border-gray-800 hover:bg-gray-800/50">
                    <td className="p-2 text-white font-medium">{balance.token_ticker}</td>
                    <td className="p-2 text-gray-300 text-right">
                      {formatSize(available.toString(), token.decimals)}
                    </td>
                    <td className="p-2 text-gray-300 text-right">
                      {formatSize(balance.open_interest, token.decimals)}
                    </td>
                    <td className="p-2 text-white font-medium text-right">
                      {formatSize(total.toString(), token.decimals)}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}
