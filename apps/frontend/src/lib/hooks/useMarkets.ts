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
        console.log('[useMarkets] Fetching markets and tokens...');
        const [marketsData, tokensData] = await Promise.all([exchange.getMarkets(), exchange.getTokens()]);

        if (mounted) {
          console.log('[useMarkets] Got data:', marketsData.length, 'markets,', tokensData.length, 'tokens');
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

          console.log('[useMarkets] Setting markets and tokens in store');
          setMarkets(enrichedMarkets);
          setTokens(tokensData);
          console.log('[useMarkets] Done!');
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
    isLoading: isLoadingMarkets,
  };
}
