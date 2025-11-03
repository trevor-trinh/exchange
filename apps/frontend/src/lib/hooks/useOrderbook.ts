/**
 * Hook for subscribing to orderbook updates
 */

import { useEffect, useMemo } from "react";
import { useExchangeStore, selectOrderbookBids, selectOrderbookAsks } from "../store";
import { useExchangeClient } from "./useExchangeClient";

export function useOrderbook(marketId: string | null) {
  const client = useExchangeClient();
  const updateOrderbook = useExchangeStore((state) => state.updateOrderbook);
  const bids = useExchangeStore(selectOrderbookBids);
  const asks = useExchangeStore(selectOrderbookAsks);

  useEffect(() => {
    if (!marketId) return;

    // Subscribe to orderbook updates using SDK convenience method
    const unsubscribe = client.onOrderbook(marketId, ({ bids, asks }) => {
      updateOrderbook(marketId, bids, asks);
    });

    // Cleanup
    return () => {
      unsubscribe();
    };
  }, [marketId, client, updateOrderbook]);

  // Calculate spread - use enhanced priceValue from SDK
  const spread = useMemo(() => {
    const lowestAsk = asks.length > 0 && asks[0] ? asks[0].priceValue : 0;
    const highestBid = bids.length > 0 && bids[0] ? bids[0].priceValue : 0;
    const spreadValue = lowestAsk && highestBid ? lowestAsk - highestBid : 0;
    const spreadPercentage = highestBid ? ((spreadValue / highestBid) * 100).toFixed(2) : "0.00";
    return { spreadValue, spreadPercentage };
  }, [asks, bids]);

  // Calculate cumulative sizes for depth visualization - use enhanced sizeValue from SDK
  const asksWithCumulative = useMemo(() => {
    return asks.slice(0, 15).map((ask, i, arr) => {
      const cumulative = arr.slice(0, i + 1).reduce((sum, a) => sum + a.sizeValue, 0);
      return { ...ask, cumulative };
    });
  }, [asks]);

  const bidsWithCumulative = useMemo(() => {
    return bids.slice(0, 15).map((bid, i, arr) => {
      const cumulative = arr.slice(0, i + 1).reduce((sum, b) => sum + b.sizeValue, 0);
      return { ...bid, cumulative };
    });
  }, [bids]);

  const maxAskCumulative = useMemo(
    () => (asksWithCumulative.length > 0 ? (asksWithCumulative[asksWithCumulative.length - 1]?.cumulative ?? 1) : 1),
    [asksWithCumulative]
  );

  const maxBidCumulative = useMemo(
    () => (bidsWithCumulative.length > 0 ? (bidsWithCumulative[bidsWithCumulative.length - 1]?.cumulative ?? 1) : 1),
    [bidsWithCumulative]
  );

  return {
    bids,
    asks,
    spread,
    asksWithCumulative,
    bidsWithCumulative,
    maxAskCumulative,
    maxBidCumulative,
  };
}
