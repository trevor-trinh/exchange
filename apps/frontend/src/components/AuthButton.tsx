"use client";

import { useEffect } from "react";
import { useTurnkey } from "@turnkey/react-wallet-kit";
import { useExchangeStore } from "@/lib/store";
import { Button } from "@/components/ui/button";
import { Copy, CheckCircle2, LogOut, Wallet } from "lucide-react";
import { toast } from "sonner";
import { useState } from "react";

export function AuthButton() {
  const { handleLogin, wallets, authState, logout } = useTurnkey();
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const setUser = useExchangeStore((state) => state.setUser);
  const clearUser = useExchangeStore((state) => state.clearUser);
  const [copied, setCopied] = useState(false);

  // Sync Turnkey auth state with our store
  useEffect(() => {
    if (authState === "authenticated" && wallets.length > 0 && !isAuthenticated) {
      // User is authenticated in Turnkey but not in our store
      const firstWallet = wallets[0];
      if (firstWallet && firstWallet.accounts && firstWallet.accounts.length > 0) {
        const address = firstWallet.accounts[0]?.address;
        if (!address) return;
        setUser(address);
      }
    } else if (authState === "unauthenticated" && isAuthenticated) {
      // User logged out from Turnkey, sync our store
      clearUser();
    }
  }, [authState, wallets, isAuthenticated, setUser, clearUser]);

  const handleLogout = () => {
    // Call Turnkey logout and clear local state
    logout();
    clearUser();
  };

  const handleCopyAddress = async () => {
    if (!userAddress) return;

    try {
      await navigator.clipboard.writeText(userAddress);
      setCopied(true);
      toast.success("Address copied to clipboard!");
      setTimeout(() => setCopied(false), 2000);
    } catch {
      toast.error("Failed to copy address");
    }
  };

  if (isAuthenticated && userAddress) {
    return (
      <div className="flex items-center gap-2">
        <Button
          size="sm"
          variant="outline"
          onClick={handleCopyAddress}
          className="gap-2 transition-all duration-200 hover:scale-[1.02] hover:shadow-md hover:bg-primary/5 hover:border-primary/50 active:scale-[0.98]"
        >
          <Wallet className="h-3.5 w-3.5 text-primary/70" />
          <span className="font-mono text-xs">
            {userAddress.slice(0, 6)}...{userAddress.slice(-4)}
          </span>
          {copied ? (
            <CheckCircle2 className="h-3.5 w-3.5 text-green-500" />
          ) : (
            <Copy className="h-3.5 w-3.5 text-muted-foreground" />
          )}
        </Button>
        <Button
          size="sm"
          variant="outline"
          onClick={handleLogout}
          className="gap-1.5 transition-all duration-200 hover:scale-[1.02] hover:shadow-md hover:bg-destructive/5 hover:border-destructive/50 hover:text-destructive active:scale-[0.98]"
        >
          <LogOut className="h-3.5 w-3.5" />
          Disconnect
        </Button>
      </div>
    );
  }

  return (
    <Button
      size="sm"
      variant="default"
      className="gap-1.5 backdrop-blur-md bg-primary/80 hover:bg-primary/90 border-b-[3px] border-b-primary shadow-[0_3px_2px_0px_rgba(180,150,255,0.6),0_1px_1px_0px_rgba(255,255,255,0.5)] cursor-pointer transition-all duration-200 hover:scale-[1.02] hover:shadow-[0_4px_6px_0px_rgba(180,150,255,0.65),0_1px_2px_0px_rgba(255,255,255,0.6)] active:scale-[0.98]"
      onClick={() => handleLogin()}
    >
      <Wallet className="h-4 w-4" />
      Connect Wallet
    </Button>
  );
}
