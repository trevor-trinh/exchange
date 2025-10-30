/**
 * Hook for subscribing to trade updates
 */

import { useEffect } from "react";
import { useExchangeStore, selectRecentTrades } from "../store";
import { getWebSocketManager } from "../websocket";
import type { TradeMessage, ServerMessage } from "../types/websocket";

export function useTrades(marketId: string | null) {
  const addTrade = useExchangeStore((state) => state.addTrade);
  const trades = useExchangeStore(selectRecentTrades);

  useEffect(() => {
    if (!marketId) return;

    const ws = getWebSocketManager();

    // Handler for trade messages
    const handleTrade = (message: TradeMessage) => {
      if (message.data.market_id === marketId) {
        addTrade({
          id: message.data.id,
          market_id: message.data.market_id,
          buyer_address: message.data.buyer_address,
          seller_address: message.data.seller_address,
          price: message.data.price,
          size: message.data.size,
          buyer_fee: "0", // Not included in WebSocket message
          seller_fee: "0", // Not included in WebSocket message
          timestamp: message.data.timestamp,
        });
      }
    };

    // Register handler
    ws.on("trade", handleTrade as (message: ServerMessage) => void);

    // Subscribe to trades
    ws.subscribe("trades", marketId);

    // Cleanup
    return () => {
      ws.off("trade", handleTrade as (message: ServerMessage) => void);
      ws.unsubscribe("trades", marketId);
    };
  }, [marketId, addTrade]);

  return trades;
}
