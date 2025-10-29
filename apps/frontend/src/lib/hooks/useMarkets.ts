/**
 * Hook for fetching and managing market data
 */

import { useEffect } from "react";
import { useExchangeStore } from "../store";
import { getAPI } from "../api";

export function useMarkets() {
  const { markets, tokens, setMarkets, setTokens, isLoadingMarkets } = useExchangeStore();
  const api = getAPI();

  useEffect(() => {
    let mounted = true;

    async function fetchData() {
      try {
        const [marketsData, tokensData] = await Promise.all([api.getMarkets(), api.getTokens()]);

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
  }, [api, setMarkets, setTokens]);

  return {
    markets,
    tokens,
    isLoading: isLoadingMarkets,
  };
}
