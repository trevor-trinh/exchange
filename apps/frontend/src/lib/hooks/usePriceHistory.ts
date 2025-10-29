/**
 * Hook for accessing price history for charts
 */

import { useExchangeStore, selectPriceHistory, selectCurrentPrice } from "../store";

export function usePriceHistory() {
  const priceHistory = useExchangeStore(selectPriceHistory);
  const currentPrice = useExchangeStore(selectCurrentPrice);

  return {
    priceHistory,
    currentPrice,
  };
}
