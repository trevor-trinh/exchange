"use client";

import { useState, useEffect } from "react";
import { useExchangeStore } from "@/lib/store";
import { getExchangeClient } from "@/lib/api";
import type { Balance } from "@/lib/types/exchange";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";

export function Balances() {
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const [balances, setBalances] = useState<Balance[]>([]);
  const [loading, setLoading] = useState(false);

  const fetchBalances = async () => {
    if (!userAddress || !isAuthenticated) {
      return;
    }

    setLoading(true);
    try {
      const client = getExchangeClient();
      const result = await client.getBalances(userAddress);
      setBalances(result);
    } catch (err) {
      console.error("Failed to fetch balances:", err);
      setBalances([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (!userAddress || !isAuthenticated) {
      setBalances([]);
      return;
    }

    fetchBalances();
    const interval = setInterval(fetchBalances, 3000); // Refresh every 3 seconds

    return () => clearInterval(interval);
  }, [userAddress, isAuthenticated]);

  return (
    <div className="space-y-6">
      <div className="rounded-lg border border-border/50 bg-card/30 backdrop-blur-sm overflow-hidden">
        {loading && !balances.length ? (
          <div className="p-8 text-center">
            <p className="text-muted-foreground text-sm">Loading balances...</p>
          </div>
        ) : !isAuthenticated || !userAddress ? (
          <div className="p-8 text-center">
            <p className="text-muted-foreground text-sm">Connect your wallet to view balances</p>
          </div>
        ) : balances.length === 0 ? (
          <div className="p-8 text-center">
            <p className="text-muted-foreground text-sm">
              No balances found. Use the faucet button in the top bar to get tokens!
            </p>
          </div>
        ) : (
          <div className="overflow-auto max-h-80">
            <Table>
              <TableHeader>
                <TableRow className="border-border/50 hover:bg-transparent">
                  <TableHead className="font-semibold text-foreground">Token</TableHead>
                  <TableHead className="text-right font-semibold text-foreground">Available</TableHead>
                  <TableHead className="text-right font-semibold text-foreground">In Orders</TableHead>
                  <TableHead className="text-right font-semibold text-foreground">Total</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {balances.map((balance) => {
                  // Calculate available (amount - locked)
                  const available = balance.amountValue - balance.lockedValue;

                  return (
                    <TableRow
                      key={balance.token_ticker}
                      className="border-border/50 hover:bg-primary/5 transition-colors"
                    >
                      <TableCell className="font-semibold text-foreground">{balance.token_ticker}</TableCell>
                      <TableCell className="text-right font-mono text-sm">{available.toFixed(8)}</TableCell>
                      <TableCell className="text-right font-mono text-sm text-muted-foreground">
                        {balance.lockedDisplay}
                      </TableCell>
                      <TableCell className="text-right font-mono text-sm font-semibold text-foreground">
                        {balance.amountDisplay}
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </div>
        )}
      </div>
    </div>
  );
}
