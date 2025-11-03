/**
 * Hook for fetching and managing market data
 */

import { useEffect } from "react";
import { useExchangeStore } from "../store";
import { useExchangeClient } from "./useExchangeClient";

export function useMarkets() {
  const client = useExchangeClient();
  const { markets, tokens, setMarkets, setTokens } = useExchangeStore();

  useEffect(() => {
    let mounted = true;

    async function fetchData() {
      try {
        // getMarkets() and getTokens() now return from cache if available
        const [marketsData, tokensData] = await Promise.all([client.getMarkets(), client.getTokens()]);

        if (mounted) {
          // Enrich markets with decimal information from tokens
          const enrichedMarkets = marketsData.map((market) => {
            const baseToken = tokensData.find((t) => t.ticker === market.base_ticker);
            const quoteToken = tokensData.find((t) => t.ticker === market.quote_ticker);

            return {
              ...market,
              base_decimals: baseToken?.decimals ?? 0,
              quote_decimals: quoteToken?.decimals ?? 0,
            };
          });

          setMarkets(enrichedMarkets);
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Only run once on mount - setMarkets/setTokens are stable but we don't need them in deps

  return {
    markets,
    tokens,
    isLoading: markets.length === 0,
  };
}
