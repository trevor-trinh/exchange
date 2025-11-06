/**
 * Hook for managing user orders with WebSocket subscriptions
 */

import { useEffect, useRef, useMemo, useCallback } from "react";
import { toast } from "sonner";
import { useExchangeStore } from "../store";
import { useExchangeClient } from "./useExchangeClient";
import { OrderStatus } from "@exchange/sdk";

/**
 * Hook that fetches initial orders via REST and subscribes to WebSocket updates
 * Orders are stored in Zustand store and automatically updated via WebSocket
 */
export function useUserOrders() {
  const client = useExchangeClient();
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const setOrders = useExchangeStore((state) => state.setOrders);
  const updateOrder = useExchangeStore((state) => state.updateOrder);
  const ordersRecord = useExchangeStore((state) => state.userOrders);

  // Convert Record to array with useMemo to avoid recreating on every render
  const orders = useMemo(() => Object.values(ordersRecord), [ordersRecord]);

  // Track if this is the initial load to avoid toasting for existing data
  const isInitialLoadRef = useRef(true);

  // Refetch orders function
  const refetchOrders = useCallback(async () => {
    if (!userAddress || !isAuthenticated) return;
    try {
      const result = await client.getOrders(userAddress, selectedMarketId || undefined);
      setOrders(result);
    } catch (err) {
      console.error("Failed to refetch orders:", err);
    }
  }, [client, userAddress, selectedMarketId, isAuthenticated, setOrders]);

  useEffect(() => {
    if (!userAddress || !isAuthenticated) {
      setOrders([]);
      isInitialLoadRef.current = true;
      return;
    }

    // Fetch initial orders via REST (returns enhanced orders from SDK)
    const fetchInitialOrders = async () => {
      try {
        const result = await client.getOrders(userAddress, selectedMarketId || undefined);
        setOrders(result);
      } catch (err) {
        console.error("Failed to fetch initial orders:", err);
        setOrders([]);
      }
    };

    fetchInitialOrders();

    // Set a short delay to mark initial load as complete
    const timer = setTimeout(() => {
      isInitialLoadRef.current = false;
    }, 2000);

    // Subscribe to WebSocket order updates
    const unsubscribe = client.onUserOrders(userAddress, async (orderUpdate) => {
      // Get latest orders from store at callback time (not from stale closure)
      const currentOrders = useExchangeStore.getState().userOrders;
      const orderExists = currentOrders[orderUpdate.order_id];

      // If order doesn't exist, refetch to get the full order details
      if (!orderExists) {
        await refetchOrders();
      } else {
        // Update existing order
        updateOrder(orderUpdate.order_id, orderUpdate.status as OrderStatus, orderUpdate.filled_size);
      }

      // Show toast notification for new order placements
      if (!isInitialLoadRef.current && orderUpdate.status === "pending" && orderUpdate.filled_size === "0") {
        // Get the full order details for a better toast message
        const currentOrders = useExchangeStore.getState().userOrders;
        const order = currentOrders[orderUpdate.order_id];

        if (order) {
          const side = order.side === "buy" ? "BUY" : "SELL";
          const market = order.market_id.split("/")[0];
          toast.success(`${side} ${order.sizeDisplay} ${market} @ ${order.priceDisplay}`, {
            description: order.order_type === "limit" ? "Limit order placed" : "Market order placed",
            duration: 3000,
          });
        }
      }

      // Show toast for cancelled orders
      if (!isInitialLoadRef.current && orderUpdate.status === "cancelled") {
        const currentOrders = useExchangeStore.getState().userOrders;
        const order = currentOrders[orderUpdate.order_id];

        if (order) {
          const side = order.side === "buy" ? "BUY" : "SELL";
          const market = order.market_id.split("/")[0];
          toast.info(`Cancelled ${side} ${order.sizeDisplay} ${market}`, {
            description: order.priceDisplay ? `@ ${order.priceDisplay}` : "Market order",
            duration: 2500,
          });
        } else {
          toast.info("Order cancelled");
        }
      }
    });

    return () => {
      clearTimeout(timer);
      unsubscribe();
    };
  }, [userAddress, isAuthenticated, selectedMarketId, client, setOrders, updateOrder, refetchOrders]);

  // Filter orders by selected market if market is selected
  return selectedMarketId ? orders.filter((o) => o.market_id === selectedMarketId) : orders;
}
