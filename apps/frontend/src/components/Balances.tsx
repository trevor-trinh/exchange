"use client";

import { useState, useEffect } from "react";
import { useExchangeStore } from "@/lib/store";
import { getExchangeClient } from "@/lib/api";
import { formatSize } from "@/lib/format";
import type { Balance } from "@exchange/sdk";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

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
        <Input
          type="text"
          value={userAddress}
          onChange={(e) => setUserAddress(e.target.value)}
          placeholder="Enter your address to view balances"
          className="max-w-md"
        />
      </div>

      <div className="overflow-auto max-h-80">
        {loading && !balances.length ? (
          <p className="text-muted-foreground text-sm">Loading balances...</p>
        ) : !userAddress.trim() ? (
          <p className="text-muted-foreground text-sm">Enter your address to view balances</p>
        ) : balances.length === 0 ? (
          <p className="text-muted-foreground text-sm">No balances found</p>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Token</TableHead>
                <TableHead className="text-right">Available</TableHead>
                <TableHead className="text-right">In Orders</TableHead>
                <TableHead className="text-right">Total</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {balances.map((balance) => {
                const token = tokens.find((t) => t.ticker === balance.token_ticker);
                if (!token) return null;

                const available = BigInt(balance.amount) - BigInt(balance.open_interest);
                const total = BigInt(balance.amount);

                return (
                  <TableRow key={balance.token_ticker}>
                    <TableCell className="font-semibold">{balance.token_ticker}</TableCell>
                    <TableCell className="text-right font-mono text-muted-foreground">
                      {formatSize(available.toString(), token.decimals)}
                    </TableCell>
                    <TableCell className="text-right font-mono text-muted-foreground">
                      {formatSize(balance.open_interest, token.decimals)}
                    </TableCell>
                    <TableCell className="text-right font-mono font-semibold">
                      {formatSize(total.toString(), token.decimals)}
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        )}
      </div>
    </div>
  );
}
