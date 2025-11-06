/**
 * Hook for managing user trades with WebSocket subscriptions
 */

import { useEffect } from "react";
import { useExchangeStore } from "../store";
import { useExchangeClient } from "./useExchangeClient";

/**
 * Hook that fetches initial user trades via REST and subscribes to WebSocket updates
 * Trades are stored in Zustand store and automatically updated via WebSocket
 */
export function useUserTrades() {
  const client = useExchangeClient();
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const setUserTrades = useExchangeStore((state) => state.setUserTrades);
  const addUserTrade = useExchangeStore((state) => state.addUserTrade);
  const userTrades = useExchangeStore((state) => state.userTrades);

  useEffect(() => {
    if (!userAddress || !isAuthenticated) {
      setUserTrades([]);
      return;
    }

    // Fetch initial trades via REST (returns enhanced trades from SDK)
    const fetchInitialTrades = async () => {
      try {
        const result = await client.getTrades(userAddress, selectedMarketId || undefined);
        // Limit to 50 most recent trades
        setUserTrades(result.slice(0, 50));
      } catch (err) {
        console.error("Failed to fetch initial trades:", err);
        setUserTrades([]);
      }
    };

    fetchInitialTrades();

    // Subscribe to WebSocket trade updates
    const unsubscribe = client.onUserFills(userAddress, (trade) => {
      addUserTrade(trade);
    });

    return () => {
      unsubscribe();
    };
  }, [userAddress, isAuthenticated, selectedMarketId, client, setUserTrades, addUserTrade]);

  // Filter trades by selected market if market is selected
  return selectedMarketId ? userTrades.filter((t) => t.market_id === selectedMarketId) : userTrades;
}
