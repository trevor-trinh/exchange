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
      if (message.trade.market_id === marketId) {
        addTrade({
          id: message.trade.id,
          market_id: message.trade.market_id,
          buyer_address: message.trade.buyer_address,
          seller_address: message.trade.seller_address,
          price: message.trade.price,
          size: message.trade.size,
          buyer_fee: "0", // Not included in WebSocket message
          seller_fee: "0", // Not included in WebSocket message
          timestamp: message.trade.timestamp,
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
