"use client";

import { useState, useEffect } from "react";
import { ThemeProvider } from "./theme-provider";
import { TurnkeyProviderWrapper } from "./turnkey-provider";
import { LoadingScreen } from "@/components/LoadingScreen";
import { useExchangeStore } from "@/lib/store";

export function Providers({ children }: { children: React.ReactNode }) {
  const [isInitialLoad, setIsInitialLoad] = useState(true);
  const [minTimeElapsed, setMinTimeElapsed] = useState(false);
  const markets = useExchangeStore((state) => state.markets);
  const tokens = useExchangeStore((state) => state.tokens);

  // Ensure loading screen shows for minimum time
  useEffect(() => {
    const timer = setTimeout(() => {
      setMinTimeElapsed(true);
    }, 600);
    return () => clearTimeout(timer);
  }, []);

  useEffect(() => {
    // Hide loading screen once we have markets and tokens loaded AND minimum time elapsed
    if (Object.keys(markets).length > 0 && Object.keys(tokens).length > 0 && minTimeElapsed) {
      // Add a small delay to ensure smooth transition
      const timer = setTimeout(() => {
        setIsInitialLoad(false);
      }, 300);
      return () => clearTimeout(timer);
    }
  }, [markets, tokens, minTimeElapsed]);

  return (
    <ThemeProvider attribute="class" defaultTheme="dark" enableSystem={false} disableTransitionOnChange>
      <TurnkeyProviderWrapper>
        <LoadingScreen isLoading={isInitialLoad} />
        {children}
      </TurnkeyProviderWrapper>
    </ThemeProvider>
  );
}
