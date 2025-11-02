"use client";

import { useState } from "react";
import { useExchangeStore } from "@/lib/store";
import { getExchangeClient } from "@/lib/api";
import { toRawValue } from "@/lib/format";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { toast } from "sonner";
import { useTurnkey } from "@turnkey/react-wallet-kit";
import { Droplet } from "lucide-react";

export function FaucetDialog() {
  const { handleLogin } = useTurnkey();
  const tokens = useExchangeStore((state) => state.tokens);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const [open, setOpen] = useState(false);
  const [loadingToken, setLoadingToken] = useState<string | null>(null);

  const handleFaucet = async (tokenTicker: string) => {
    if (!userAddress) {
      return;
    }

    setLoadingToken(tokenTicker);

    try {
      const client = getExchangeClient();
      const token = tokens.find((t) => t.ticker === tokenTicker);
      if (!token) {
        throw new Error("Token not found");
      }

      // Faucet 1000 tokens (adjust amount with decimals)
      const amount = toRawValue(1000, token.decimals);

      await client.rest.faucet({
        userAddress,
        tokenTicker,
        amount,
        signature: `${userAddress}:${Date.now()}`,
      });

      toast.success(`Successfully received 1000 ${tokenTicker}!`, {
        description: "Your balance has been updated",
      });
    } catch (err) {
      console.error("Faucet error:", err);
      toast.error("Failed to get tokens", {
        description: err instanceof Error ? err.message : "Please try again later",
      });
    } finally {
      setLoadingToken(null);
    }
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button size="sm" variant="outline" className="gap-2">
          <Droplet className="h-4 w-4" />
          Faucet
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Token Faucet</DialogTitle>
          <DialogDescription>
            {isAuthenticated
              ? "Select a token to receive 1000 tokens for testing"
              : "Connect your wallet to use the faucet"}
          </DialogDescription>
        </DialogHeader>

        {!isAuthenticated ? (
          <div className="flex flex-col items-center gap-4 py-6">
            <p className="text-sm text-muted-foreground text-center">
              You need to connect your wallet before you can use the faucet
            </p>
            <Button
              onClick={() => {
                setOpen(false);
                handleLogin();
              }}
              className="backdrop-blur-md bg-primary/80 hover:bg-primary/90"
            >
              Connect Wallet
            </Button>
          </div>
        ) : (
          <div className="grid gap-3 py-4">
            {tokens.map((token) => (
              <div
                key={token.ticker}
                className="flex items-center justify-between p-3 rounded-lg border border-border/50 bg-muted/30 hover:bg-muted/50 transition-colors"
              >
                <div>
                  <div className="font-semibold">{token.ticker}</div>
                  <div className="text-xs text-muted-foreground">{token.name}</div>
                </div>
                <Button
                  size="sm"
                  onClick={() => handleFaucet(token.ticker)}
                  disabled={loadingToken !== null}
                  className="bg-primary hover:bg-primary/90"
                >
                  {loadingToken === token.ticker ? "Getting..." : "Get 1000"}
                </Button>
              </div>
            ))}
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
