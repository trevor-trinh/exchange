"use client";

import { useState, useEffect, useRef, useMemo } from "react";
import { useExchangeStore } from "@/lib/store";
import { useExchangeClient } from "@/lib/hooks/useExchangeClient";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { toast } from "sonner";
import { useTurnkey } from "@turnkey/react-wallet-kit";
import { Droplet } from "lucide-react";

interface FaucetDialogProps {
  /** If true, dialog is controlled externally via open/onOpenChange */
  controlled?: boolean;
  /** External open state (only used when controlled=true) */
  open?: boolean;
  /** External open change handler (only used when controlled=true) */
  onOpenChange?: (open: boolean) => void;
  /** Custom trigger element (only used when controlled=false) */
  trigger?: React.ReactNode;
}

export function FaucetDialog({
  controlled = false,
  open: externalOpen,
  onOpenChange: externalOnOpenChange,
  trigger,
}: FaucetDialogProps) {
  const client = useExchangeClient();
  const { handleLogin } = useTurnkey();
  const tokensRecord = useExchangeStore((state) => state.tokens);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);

  // Convert Record to array with useMemo to avoid recreating on every render
  const tokens = useMemo(() => Object.values(tokensRecord), [tokensRecord]);
  const [internalOpen, setInternalOpen] = useState(false);
  const [selectedToken, setSelectedToken] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const waitingForAuthRef = useRef(false);

  // Use controlled or internal state
  const open = controlled ? (externalOpen ?? false) : internalOpen;
  const setOpen = useMemo(
    () => (controlled ? (externalOnOpenChange ?? (() => {})) : setInternalOpen),
    [controlled, externalOnOpenChange, setInternalOpen]
  );

  // Reopen faucet after successful wallet connection
  useEffect(() => {
    if (isAuthenticated && waitingForAuthRef.current) {
      waitingForAuthRef.current = false;
      setOpen(true);
    }
  }, [isAuthenticated, setOpen]);

  const handleFaucet = async () => {
    if (!userAddress || !selectedToken) {
      return;
    }

    setLoading(true);

    try {
      // Use SDK's faucetDecimal - it handles conversion to atoms
      await client.rest.faucetDecimal({
        userAddress,
        tokenTicker: selectedToken,
        amountDecimal: 1000,
        signature: `${userAddress}:${Date.now()}`,
      });

      toast.success(`Successfully received 1000 ${selectedToken}!`, {
        description: "Your balance has been updated",
      });

      // Reset selection after success
      setSelectedToken("");
    } catch (err) {
      console.error("Faucet error:", err);
      toast.error("Failed to get tokens", {
        description: err instanceof Error ? err.message : "Please try again later",
      });
    } finally {
      setLoading(false);
    }
  };

  // Default trigger button
  const defaultTrigger = (
    <Button
      size="sm"
      variant="outline"
      className="gap-2 transition-all duration-200 hover:scale-[1.02] hover:shadow-md hover:bg-primary/5 hover:border-primary/50 active:scale-[0.98]"
    >
      <Droplet className="h-4 w-4" />
      Faucet
    </Button>
  );

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      {!controlled && <DialogTrigger asChild>{trigger ?? defaultTrigger}</DialogTrigger>}
      <DialogContent
        className="sm:max-w-md bg-card/95 backdrop-blur-xl border-border/50"
        style={{
          backgroundImage: `
            url("data:image/svg+xml,%3Csvg width='4' height='4' viewBox='0 0 4 4' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M0 0h1v1H0zM2 2h1v1H2z' fill='%23000000' fill-opacity='0.1'/%3E%3Cpath d='M1 0h1v1H1zM3 2h1v1H3zM0 2h1v1H0zM2 0h1v1H2zM1 2h1v1H1zM3 0h1v1H3z' fill='%23ffffff' fill-opacity='0.05'/%3E%3C/svg%3E"),
            repeating-linear-gradient(0deg, rgba(0, 0, 0, 0.03) 0px, rgba(0, 0, 0, 0.03) 1px, transparent 1px, transparent 2px)
          `,
          backgroundBlendMode: "overlay, normal",
        }}
      >
        <DialogHeader>
          <DialogTitle className="text-xl text-foreground">Token Faucet</DialogTitle>
          <DialogDescription className="text-muted-foreground">
            {isAuthenticated
              ? "Choose a token and get 1000 tokens for testing"
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
                waitingForAuthRef.current = true;
                setOpen(false);
                handleLogin();
              }}
              className="bg-primary/90 hover:bg-primary border border-primary/30 shadow-lg transition-all"
            >
              Connect Wallet
            </Button>
          </div>
        ) : (
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="token-select" className="text-sm font-medium">
                Select Token
              </Label>
              <Select value={selectedToken} onValueChange={setSelectedToken}>
                <SelectTrigger
                  id="token-select"
                  className="w-full bg-background/60 border-border/40 hover:bg-background/80 hover:border-border/60 transition-colors"
                >
                  <SelectValue placeholder="Choose a token..." />
                </SelectTrigger>
                <SelectContent className="bg-muted/95 backdrop-blur-xl border-border/50">
                  {tokens.map((token) => (
                    <SelectItem key={token.ticker} value={token.ticker} className="hover:bg-accent/50">
                      <div className="flex items-center gap-2">
                        <span className="font-semibold">{token.ticker}</span>
                        <span className="text-muted-foreground text-xs">- {token.name}</span>
                      </div>
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            <Button
              onClick={handleFaucet}
              disabled={!selectedToken || loading}
              className="w-full bg-linear-to-br from-primary/90 to-primary/70 hover:from-primary hover:to-primary/80 shadow-lg hover:shadow-xl border border-primary/30 transition-all gap-2"
              size="lg"
            >
              <Droplet className="h-4 w-4" />
              {loading ? "Getting Tokens..." : "Get 1000 Tokens"}
            </Button>
          </div>
        )}
      </DialogContent>
    </Dialog>
  );
}
