/**
 * Hook for subscribing to trade updates
 */

import { useEffect } from "react";
import { useExchangeStore, selectRecentTrades } from "../store";
import { getExchangeClient } from "../api";

export function useTrades(marketId: string | null) {
  const addTrade = useExchangeStore((state) => state.addTrade);
  const trades = useExchangeStore(selectRecentTrades);

  useEffect(() => {
    if (!marketId) return;

    const client = getExchangeClient();

    // Subscribe to trade updates using SDK convenience method
    // SDK now returns fully enhanced trades! 🎉
    const unsubscribe = client.onTrades(marketId, (enhancedTrade) => {
      // SDK already enhanced the trade, just add it to store
      addTrade(enhancedTrade);
    });

    // Cleanup
    return unsubscribe;
  }, [marketId, addTrade]);

  return trades;
}
