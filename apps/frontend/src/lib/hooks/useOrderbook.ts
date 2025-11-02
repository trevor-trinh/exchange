/**
 * Hook for subscribing to orderbook updates
 */

import { useEffect } from "react";
import { useExchangeStore, selectOrderbookBids, selectOrderbookAsks } from "../store";
import { getExchangeClient } from "../api";

export function useOrderbook(marketId: string | null) {
  const updateOrderbook = useExchangeStore((state) => state.updateOrderbook);
  const setOrderbookLoading = useExchangeStore((state) => state.setOrderbookLoading);
  const bids = useExchangeStore(selectOrderbookBids);
  const asks = useExchangeStore(selectOrderbookAsks);

  useEffect(() => {
    if (!marketId) return;

    console.log('[useOrderbook] Subscribing to orderbook for', marketId);
    const client = getExchangeClient();
    setOrderbookLoading(true);

    // Subscribe to orderbook updates using SDK convenience method
    const unsubscribe = client.onOrderbook(marketId, ({ bids, asks }) => {
      updateOrderbook(marketId, bids, asks);
    });

    // Cleanup
    return () => {
      console.log('[useOrderbook] Cleaning up subscription for', marketId);
      unsubscribe();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [marketId]); // Only re-subscribe when marketId changes - store functions are stable

  return {
    bids,
    asks,
  };
}
