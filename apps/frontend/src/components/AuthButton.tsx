"use client";

import { useState, useEffect } from "react";
import { useTurnkey } from "@turnkey/sdk-react";
import { useExchangeStore } from "@/lib/store";
import { autoFaucet } from "@/lib/faucet";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export function AuthButton() {
  const { turnkey, passkeyClient, authIframeClient } = useTurnkey();
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const setUser = useExchangeStore((state) => state.setUser);
  const clearUser = useExchangeStore((state) => state.clearUser);
  const tokens = useExchangeStore((state) => state.tokens);

  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [email, setEmail] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Try to restore session on mount
  useEffect(() => {
    const checkSession = async () => {
      if (!authIframeClient) return;

      try {
        const session = await authIframeClient.getWhoami();
        if (session?.organizationId && session?.userId) {
          // Get the user's wallet address from their first wallet
          const wallets = await authIframeClient.getWallets();
          if (wallets && wallets.length > 0) {
            const walletAddress = wallets[0].walletId;
            setUser(walletAddress);

            // Auto-faucet for returning users (check if they need tokens)
            if (tokens.length > 0) {
              await autoFaucet(walletAddress, tokens);
            }
          }
        }
      } catch (err) {
        console.log("No existing session");
      }
    };
    checkSession();
  }, [authIframeClient, setUser, tokens]);

  const handleEmailAuth = async () => {
    if (!email.trim()) {
      setError("Please enter your email");
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      if (!authIframeClient) {
        throw new Error("Turnkey not initialized. Make sure NEXT_PUBLIC_TURNKEY_ORGANIZATION_ID is set.");
      }

      // Initiate email auth - this will send an email with a magic link
      await authIframeClient.injectCredentialBundle(email.trim());

      // After email verification, create/get wallet
      const wallets = await authIframeClient.getWallets();

      if (wallets && wallets.length > 0) {
        const walletAddress = wallets[0].walletId;
        setUser(walletAddress);
        setIsDialogOpen(false);
        setEmail("");

        // Auto-faucet for new users
        if (tokens.length > 0) {
          await autoFaucet(walletAddress, tokens);
        }
      } else {
        // Create a new wallet for the user
        const newWallet = await authIframeClient.createWallet({
          walletName: `Wallet ${Date.now()}`,
        });

        if (newWallet?.walletId) {
          setUser(newWallet.walletId);
          setIsDialogOpen(false);
          setEmail("");

          // Auto-faucet for new users
          if (tokens.length > 0) {
            await autoFaucet(newWallet.walletId, tokens);
          }
        }
      }
    } catch (err) {
      console.error("Auth error:", err);
      setError(err instanceof Error ? err.message : "Authentication failed");
    } finally {
      setIsLoading(false);
    }
  };

  const handlePasskeyAuth = async () => {
    setIsLoading(true);
    setError(null);

    try {
      if (!passkeyClient) {
        throw new Error("Passkey client not initialized. Make sure NEXT_PUBLIC_TURNKEY_ORGANIZATION_ID is set.");
      }

      // Attempt passkey login
      const result = await passkeyClient.login();

      if (result?.walletId) {
        setUser(result.walletId);
        setIsDialogOpen(false);

        // Auto-faucet for users
        if (tokens.length > 0) {
          await autoFaucet(result.walletId, tokens);
        }
      }
    } catch (err) {
      console.error("Passkey auth error:", err);
      setError(err instanceof Error ? err.message : "Passkey authentication failed. Try creating an account with email first.");
    } finally {
      setIsLoading(false);
    }
  };

  const handleLogout = () => {
    clearUser();
  };

  if (isAuthenticated && userAddress) {
    return (
      <div className="flex items-center gap-3">
        <div className="text-sm">
          <span className="text-muted-foreground">Connected: </span>
          <span className="font-mono text-xs">
            {userAddress.slice(0, 6)}...{userAddress.slice(-4)}
          </span>
        </div>
        <Button size="sm" variant="outline" onClick={handleLogout}>
          Disconnect
        </Button>
      </div>
    );
  }

  return (
    <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
      <DialogTrigger asChild>
        <Button
          size="sm"
          variant="default"
          className="backdrop-blur-md bg-primary/80 hover:bg-primary/90 border-b-[3px] border-b-primary shadow-[0_2px_1px_0px_rgba(180,150,255,0.6),0_1px_2px_0px_rgba(255,255,255,0.5)] cursor-pointer"
        >
          Connect Wallet
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Connect Your Wallet</DialogTitle>
          <DialogDescription>
            Sign in with email or passkey to create or access your embedded wallet
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Email Authentication */}
          <div className="space-y-2">
            <Label htmlFor="email">Email</Label>
            <div className="flex gap-2">
              <Input
                id="email"
                type="email"
                placeholder="your@email.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") handleEmailAuth();
                }}
                disabled={isLoading}
              />
              <Button onClick={handleEmailAuth} disabled={isLoading}>
                {isLoading ? "Sending..." : "Continue"}
              </Button>
            </div>
            <p className="text-xs text-muted-foreground">
              We'll send a magic link to your email
            </p>
          </div>

          <div className="relative">
            <div className="absolute inset-0 flex items-center">
              <span className="w-full border-t" />
            </div>
            <div className="relative flex justify-center text-xs uppercase">
              <span className="bg-background px-2 text-muted-foreground">Or</span>
            </div>
          </div>

          {/* Passkey Authentication */}
          <Button
            onClick={handlePasskeyAuth}
            disabled={isLoading}
            variant="outline"
            className="w-full"
          >
            Sign in with Passkey
          </Button>
          <p className="text-xs text-muted-foreground text-center">
            Passkeys use your device's biometrics for secure authentication
          </p>

          {error && (
            <div className="rounded-md bg-destructive/15 p-3 text-sm text-destructive">
              {error}
            </div>
          )}

          {!turnkey && (
            <div className="bg-yellow-500/10 border border-yellow-500/20 p-3 text-sm text-yellow-600 dark:text-yellow-500 rounded-md">
              <p className="font-semibold mb-1">Configuration Required</p>
              <p className="text-xs">
                Add your Turnkey organization ID to .env.local to enable authentication
              </p>
            </div>
          )}
        </div>
      </DialogContent>
    </Dialog>
  );
}
