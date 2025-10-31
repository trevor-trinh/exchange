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
          // Enrich markets with decimal information from tokens
          const enrichedMarkets = marketsData.map((market: any) => {
            const baseToken = tokensData.find((t: any) => t.ticker === market.base_ticker);
            const quoteToken = tokensData.find((t: any) => t.ticker === market.quote_ticker);

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
  }, [setMarkets, setTokens]);

  return {
    markets,
    tokens,
    isLoading: isLoadingMarkets,
  };
}
