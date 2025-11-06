"use client";

import { useMemo, useCallback } from "react";
import { ColumnDef } from "@tanstack/react-table";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { useUserOrders, useCancelOrder } from "@/lib/hooks";
import type { Order } from "@/lib/types/exchange";
import { DataTable } from "@/components/ui/data-table";
import { Button } from "@/components/ui/button";
import { X } from "lucide-react";

export function RecentOrders() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const orders = useUserOrders();
  const { cancelOrder, cancelAllOrders, cancellingOrders, cancellingAll } = useCancelOrder();

  const handleCancelOrder = useCallback(
    async (orderId: string) => {
      if (!userAddress) return;
      try {
        await cancelOrder(userAddress, orderId);
      } catch {
        // Error is already logged in the hook
      }
    },
    [userAddress, cancelOrder]
  );

  const handleCancelAll = useCallback(async () => {
    if (!userAddress) return;
    try {
      await cancelAllOrders(userAddress, selectedMarketId || undefined);
    } catch {
      // Error is already logged in the hook
    }
  }, [userAddress, selectedMarketId, cancelAllOrders]);

  const hasOpenOrders = orders.some((o) => o.status === "pending" || o.status === "partially_filled");

  const columns = useMemo<ColumnDef<Order>[]>(
    () => [
      {
        accessorKey: "created_at",
        header: "Time",
        cell: ({ row }) => (
          <div className="text-muted-foreground/80 text-xs">
            {(row.getValue("created_at") as Date).toLocaleTimeString()}
          </div>
        ),
        size: 90,
      },
      {
        accessorKey: "market_id",
        header: "Market",
        cell: ({ row }) => <div className="font-medium text-foreground/90">{row.getValue("market_id")}</div>,
        size: 100,
      },
      {
        accessorKey: "side",
        header: "Side",
        cell: ({ row }) => (
          <span className={`font-semibold ${row.getValue("side") === "buy" ? "text-green-500" : "text-red-500"}`}>
            {row.getValue("side") === "buy" ? "Buy" : "Sell"}
          </span>
        ),
        size: 70,
      },
      {
        accessorKey: "order_type",
        header: "Type",
        cell: ({ row }) => (
          <span className="text-muted-foreground/80">
            {row.getValue("order_type") === "limit" ? "Limit" : "Market"}
          </span>
        ),
        size: 70,
      },
      {
        accessorKey: "priceDisplay",
        header: () => <div className="text-right">Price</div>,
        cell: ({ row }) => (
          <div className="text-right font-medium text-foreground/90">{row.getValue("priceDisplay")}</div>
        ),
        size: 120,
      },
      {
        accessorKey: "sizeDisplay",
        header: () => <div className="text-right">Size</div>,
        cell: ({ row }) => <div className="text-right text-muted-foreground/80">{row.getValue("sizeDisplay")}</div>,
        size: 120,
      },
      {
        id: "usdValue",
        header: () => <div className="text-right">USD Value</div>,
        cell: ({ row }) => {
          const order = row.original;
          const usdValue = order.priceValue * order.sizeValue;
          return (
            <div className="text-right font-medium text-foreground/90">
              ${usdValue.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
            </div>
          );
        },
        size: 120,
      },
      {
        id: "filled",
        header: () => <div className="text-right">Filled</div>,
        cell: ({ row }) => {
          const order = row.original;
          const filledPercent = order.sizeValue > 0 ? (order.filledValue / order.sizeValue) * 100 : 0;
          return <div className="text-right text-muted-foreground/80">{filledPercent.toFixed(1)}%</div>;
        },
        size: 80,
      },
      {
        accessorKey: "status",
        header: () => <div className="text-center">Status</div>,
        cell: ({ row }) => {
          const status = row.getValue("status") as string;
          return (
            <div className="flex justify-center">
              <span
                className={`inline-flex items-center text-xs px-2 py-1 font-medium rounded ${
                  status === "filled"
                    ? "bg-green-500/10 text-green-500 border border-green-500/20"
                    : status === "partially_filled"
                      ? "bg-yellow-500/10 text-yellow-500 border border-yellow-500/20"
                      : status === "cancelled"
                        ? "bg-muted text-muted-foreground/70 border border-border/40"
                        : status === "pending"
                          ? "bg-blue-500/10 text-blue-500 border border-blue-500/20"
                          : "bg-gray-500/10 text-gray-500 border border-gray-500/20"
                }`}
              >
                {status === "pending"
                  ? "Open"
                  : status === "partially_filled"
                    ? "Partial"
                    : status.charAt(0).toUpperCase() + status.slice(1)}
              </span>
            </div>
          );
        },
        size: 80,
      },
      {
        id: "actions",
        header: () => (
          <div className="flex justify-center">
            <Button
              variant="outline"
              size="sm"
              onClick={handleCancelAll}
              disabled={!hasOpenOrders || cancellingAll}
              className="text-red-500 hover:text-red-600 hover:bg-red-500/10 h-7 disabled:text-muted-foreground disabled:hover:bg-transparent disabled:pointer-events-auto disabled:cursor-not-allowed"
            >
              {cancellingAll ? "Cancelling..." : "Cancel All"}
            </Button>
          </div>
        ),
        cell: ({ row }) => {
          const order = row.original;
          const canCancel = order.status === "pending" || order.status === "partially_filled";
          const isCancelling = cancellingOrders.has(order.id);

          return (
            <div className="flex justify-center">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => handleCancelOrder(order.id)}
                disabled={!canCancel || isCancelling}
                className="h-7 w-7 p-0 text-muted-foreground hover:text-red-500 disabled:opacity-30 disabled:pointer-events-auto disabled:cursor-not-allowed cursor-pointer disabled:hover:bg-transparent disabled:hover:text-muted-foreground"
              >
                <X className="h-4 w-4" />
              </Button>
            </div>
          );
        },
        size: 80,
      },
    ],
    [cancellingOrders, hasOpenOrders, cancellingAll, handleCancelAll, handleCancelOrder]
  );

  if (!selectedMarketId || !selectedMarket) {
    return (
      <div className="h-full flex items-center justify-center">
        <p className="text-muted-foreground text-sm">Select a market to view orders</p>
      </div>
    );
  }

  if (!isAuthenticated || !userAddress) {
    return (
      <div className="h-full flex pt-20 justify-center">
        <p className="text-muted-foreground text-sm">Connect your wallet to view orders</p>
      </div>
    );
  }

  return (
    <div className="h-full">
      <DataTable columns={columns} data={orders} emptyMessage="No orders found" />
    </div>
  );
}
