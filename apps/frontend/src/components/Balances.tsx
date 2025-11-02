"use client";

import { useState, useEffect } from "react";
import { useExchangeStore } from "@/lib/store";
import { getExchangeClient } from "@/lib/api";
import { formatSize, toRawValue } from "@/lib/format";
import type { Balance } from "@exchange/sdk";
import { Button } from "@/components/ui/button";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

export function Balances() {
  const tokens = useExchangeStore((state) => state.tokens);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const [balances, setBalances] = useState<Balance[]>([]);
  const [loading, setLoading] = useState(false);
  const [faucetToken, setFaucetToken] = useState<string>("");
  const [faucetLoading, setFaucetLoading] = useState(false);
  const [faucetMessage, setFaucetMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

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

  const handleFaucet = async () => {
    if (!faucetToken || !userAddress) {
      return;
    }

    setFaucetLoading(true);
    setFaucetMessage(null);

    try {
      const client = getExchangeClient();
      const token = tokens.find((t) => t.ticker === faucetToken);
      if (!token) {
        throw new Error("Token not found");
      }

      // Faucet 1000 tokens (adjust amount with decimals)
      const amount = toRawValue(1000, token.decimals);

      await client.rest.faucet({
        userAddress,
        tokenTicker: faucetToken,
        amount,
        signature: `${userAddress}:${Date.now()}`,
      });

      setFaucetMessage({ type: "success", text: `Successfully received 1000 ${faucetToken}!` });

      // Refresh balances
      setTimeout(fetchBalances, 500);
    } catch (err) {
      console.error("Faucet error:", err);
      setFaucetMessage({
        type: "error",
        text: err instanceof Error ? err.message : "Failed to get tokens from faucet",
      });
    } finally {
      setFaucetLoading(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Faucet UI */}
      {isAuthenticated && userAddress && (
        <div className="flex flex-col sm:flex-row items-start sm:items-center gap-3 p-4 bg-muted/30 border border-border/50 rounded-lg backdrop-blur-sm">
          <div className="flex items-center gap-2 flex-1 w-full sm:w-auto">
            <Select value={faucetToken} onValueChange={setFaucetToken}>
              <SelectTrigger className="w-full sm:w-[180px] bg-card/100">
                <SelectValue placeholder="Select token" />
              </SelectTrigger>
              <SelectContent className="bg-card/100 backdrop-blur-sm">
                {tokens.map((token) => (
                  <SelectItem key={token.ticker} value={token.ticker}>
                    {token.ticker}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              onClick={handleFaucet}
              disabled={!faucetToken || faucetLoading}
              size="sm"
              className="bg-primary hover:bg-primary/90"
            >
              {faucetLoading ? "Getting tokens..." : "Get 1000 tokens"}
            </Button>
          </div>
          {faucetMessage && (
            <div
              className={`text-xs px-3 py-2 rounded-md font-medium ${
                faucetMessage.type === "success"
                  ? "bg-green-500/10 text-green-500 border border-green-500/20"
                  : "bg-red-500/10 text-red-500 border border-red-500/20"
              }`}
            >
              {faucetMessage.text}
            </div>
          )}
        </div>
      )}

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
            <p className="text-muted-foreground text-sm">No balances found. Use the faucet above to get tokens!</p>
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
                  const token = tokens.find((t) => t.ticker === balance.token_ticker);
                  if (!token) return null;

                  const available = BigInt(balance.amount) - BigInt(balance.open_interest);
                  const total = BigInt(balance.amount);

                  return (
                    <TableRow
                      key={balance.token_ticker}
                      className="border-border/50 hover:bg-primary/5 transition-colors"
                    >
                      <TableCell className="font-semibold text-foreground">{balance.token_ticker}</TableCell>
                      <TableCell className="text-right font-mono text-sm">
                        {formatSize(available.toString(), token.decimals)}
                      </TableCell>
                      <TableCell className="text-right font-mono text-sm text-muted-foreground">
                        {formatSize(balance.open_interest, token.decimals)}
                      </TableCell>
                      <TableCell className="text-right font-mono text-sm font-semibold text-foreground">
                        {formatSize(total.toString(), token.decimals)}
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
