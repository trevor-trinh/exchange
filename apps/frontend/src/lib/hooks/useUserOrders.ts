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
  // Use store-level toast tracking to prevent duplicates across all hook instances
  const markEventToasted = useExchangeStore((state) => state.markEventToasted);
  const toastedEvents = useExchangeStore((state) => state.toastedEvents);

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
      const orderExists = ordersRecord[orderUpdate.order_id];

      // If order doesn't exist, refetch to get the full order details
      if (!orderExists) {
        await refetchOrders();
      } else {
        // Update existing order
        updateOrder(orderUpdate.order_id, orderUpdate.status as OrderStatus, orderUpdate.filled_size);
      }

      // Show toast notification for new order placements (only once per order globally)
      const placeEventId = `order-placed-${orderUpdate.order_id}`;
      if (
        !isInitialLoadRef.current &&
        orderUpdate.status === "pending" &&
        orderUpdate.filled_size === "0" &&
        !toastedEvents.has(placeEventId)
      ) {
        markEventToasted(placeEventId);
        toast.info("Order placed successfully", {
          description: `Order ID: ${orderUpdate.order_id.slice(0, 8)}...`,
          duration: 3000,
        });
      }

      // Show toast for cancelled orders
      const cancelEventId = `order-cancelled-${orderUpdate.order_id}`;
      if (
        !isInitialLoadRef.current &&
        orderUpdate.status === "cancelled" &&
        !toastedEvents.has(cancelEventId)
      ) {
        markEventToasted(cancelEventId);
        toast.info("Order cancelled", {
          description: `Order ID: ${orderUpdate.order_id.slice(0, 8)}...`,
          duration: 3000,
        });
      }
    });

    return () => {
      clearTimeout(timer);
      unsubscribe();
    };
  }, [userAddress, isAuthenticated, selectedMarketId, client, setOrders, updateOrder, ordersRecord, refetchOrders, toastedEvents, markEventToasted]);

  // Filter orders by selected market if market is selected
  return selectedMarketId ? orders.filter((o) => o.market_id === selectedMarketId) : orders;
}
