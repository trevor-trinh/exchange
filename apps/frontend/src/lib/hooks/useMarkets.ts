/**
 * Hook for fetching and managing market data
 */

import { useEffect } from "react";
import { useExchangeStore } from "../store";
import { exchange } from "../api";

export function useMarkets() {
  const { markets, tokens, setMarkets, setTokens, isLoadingMarkets } = useExchangeStore();

  useEffect(() => {
    let mounted = true;

    async function fetchData() {
      try {
        const [marketsData, tokensData] = await Promise.all([exchange.getMarkets(), exchange.getTokens()]);

        if (mounted) {
          setMarkets(marketsData);
          setTokens(tokensData);
        }
      } catch (error) {
        console.error("Failed to fetch markets:", error);
      }
    }

    fetchData();

    return () => {
      mounted = false;
    };
  }, [setMarkets, setTokens]);

  return {
    markets,
    tokens,
    isLoading: isLoadingMarkets,
  };
}
