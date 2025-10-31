/**
 * Hook for subscribing to orderbook updates
 */

import { useEffect } from "react";
import { useExchangeStore, selectOrderbookBids, selectOrderbookAsks } from "../store";
import { getWebSocketManager } from "../websocket";
import type { ServerMessage } from "../types/websocket";

export function useOrderbook(marketId: string | null) {
  const updateOrderbook = useExchangeStore((state) => state.updateOrderbook);
  const setOrderbookLoading = useExchangeStore((state) => state.setOrderbookLoading);
  const bids = useExchangeStore(selectOrderbookBids);
  const asks = useExchangeStore(selectOrderbookAsks);

  useEffect(() => {
    if (!marketId) return;

    const ws = getWebSocketManager();
    setOrderbookLoading(true);

    // Handler for orderbook messages
    const handleOrderbook = (message: any) => {
      if (message.orderbook && message.orderbook.market_id === marketId) {
        updateOrderbook(message.orderbook.market_id, message.orderbook.bids, message.orderbook.asks);
      }
    };

    // Register handler
    ws.on("orderbook", handleOrderbook as (message: ServerMessage) => void);

    // Subscribe to orderbook
    ws.subscribe("orderbook", marketId);

    // Cleanup
    return () => {
      ws.off("orderbook", handleOrderbook as (message: ServerMessage) => void);
      ws.unsubscribe("orderbook", marketId);
    };
  }, [marketId, updateOrderbook, setOrderbookLoading]);

  return {
    bids,
    asks,
  };
}
