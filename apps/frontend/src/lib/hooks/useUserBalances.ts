/**
 * Hook for managing user balances with WebSocket subscriptions
 */

import { useEffect, useMemo } from "react";
import { useExchangeStore } from "../store";
import { useExchangeClient } from "./useExchangeClient";

/**
 * Hook that fetches initial balances via REST and subscribes to WebSocket updates
 * Balances are stored in Zustand store and automatically updated via WebSocket
 */
export function useUserBalances() {
  const client = useExchangeClient();
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const setBalances = useExchangeStore((state) => state.setBalances);
  const updateBalance = useExchangeStore((state) => state.updateBalance);
  const balancesRecord = useExchangeStore((state) => state.userBalances);

  // Convert Record to array with useMemo to avoid recreating on every render
  const balances = useMemo(() => Object.values(balancesRecord), [balancesRecord]);

  useEffect(() => {
    if (!userAddress || !isAuthenticated) {
      setBalances([]);
      return;
    }

    // Fetch initial balances via REST (returns enhanced balances from SDK)
    const fetchInitialBalances = async () => {
      try {
        const result = await client.getBalances(userAddress);
        setBalances(result);
      } catch (err) {
        console.error("Failed to fetch initial balances:", err);
        setBalances([]);
      }
    };

    fetchInitialBalances();

    // Subscribe to WebSocket balance updates
    const unsubscribe = client.onUserBalances(userAddress, (balanceUpdate) => {
      updateBalance(balanceUpdate.token_ticker, balanceUpdate.available, balanceUpdate.locked);
    });

    return () => {
      unsubscribe();
    };
  }, [userAddress, isAuthenticated, client, setBalances, updateBalance]);

  return balances;
}
