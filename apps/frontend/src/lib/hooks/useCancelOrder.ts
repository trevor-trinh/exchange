/**
 * Hook for canceling orders
 */

import { useState, useCallback } from "react";
import { useExchangeClient } from "./useExchangeClient";

export function useCancelOrder() {
  const client = useExchangeClient();
  const [cancellingOrders, setCancellingOrders] = useState<Set<string>>(new Set());
  const [cancellingAll, setCancellingAll] = useState(false);

  const cancelOrder = useCallback(
    async (userAddress: string, orderId: string) => {
      if (!userAddress) return;

      setCancellingOrders((prev) => new Set(prev).add(orderId));
      try {
        // TODO: Get signature from wallet
        await client.cancelOrder({
          userAddress,
          orderId,
          signature: "0x", // Placeholder - need wallet integration
        });
      } catch (err) {
        console.error("Failed to cancel order:", err);
        throw err;
      } finally {
        setCancellingOrders((prev) => {
          const next = new Set(prev);
          next.delete(orderId);
          return next;
        });
      }
    },
    [client]
  );

  const cancelAllOrders = useCallback(
    async (userAddress: string, marketId?: string) => {
      if (!userAddress) return;

      setCancellingAll(true);
      try {
        // TODO: Get signature from wallet
        await client.cancelAllOrders({
          userAddress,
          marketId: marketId || undefined,
          signature: "0x", // Placeholder - need wallet integration
        });
      } catch (err) {
        console.error("Failed to cancel all orders:", err);
        throw err;
      } finally {
        setCancellingAll(false);
      }
    },
    [client]
  );

  return {
    cancelOrder,
    cancelAllOrders,
    cancellingOrders,
    cancellingAll,
  };
}
