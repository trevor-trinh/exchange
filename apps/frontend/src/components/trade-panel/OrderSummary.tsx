import { formatNumberWithCommas } from "@/lib/format";
import type { Token } from "@/lib/types/exchange";
import type { OrderSide, OrderEstimate } from "./types";

interface OrderSummaryProps {
  estimate: OrderEstimate | null;
  side: OrderSide;
  quoteToken: Token;
  priceDecimals: number;
  feeBps: number;
}

export function OrderSummary({ estimate, side, quoteToken, priceDecimals, feeBps }: OrderSummaryProps) {
  if (!estimate || estimate.size <= 0 || estimate.price <= 0) {
    return null;
  }

  const displayDecimals = Math.min(priceDecimals, 4);

  return (
    <div className="space-y-1.5 bg-muted/30 border border-border/40 rounded-md p-3 text-xs">
      <div className="flex justify-between items-center gap-2 text-muted-foreground">
        <span className="shrink-0">Total</span>
        <span className="font-mono text-right truncate min-w-0 whitespace-nowrap">
          {formatNumberWithCommas(estimate.total, displayDecimals)}{" "}
          <span className="text-[10px]">{quoteToken.ticker}</span>
        </span>
      </div>
      <div className="flex justify-between items-center gap-2 text-muted-foreground">
        <span className="shrink-0">Fee ({(Math.abs(feeBps) / 100).toFixed(2)}%)</span>
        <span className="font-mono text-right truncate min-w-0 whitespace-nowrap">
          {formatNumberWithCommas(estimate.fee, displayDecimals)}{" "}
          <span className="text-[10px]">{quoteToken.ticker}</span>
        </span>
      </div>
      <div className="flex justify-between items-center gap-2 pt-1.5 border-t border-border/40">
        <span className="font-medium shrink-0">{side === "buy" ? "You Pay" : "You Get"}</span>
        <span className="font-mono font-semibold text-right truncate min-w-0 whitespace-nowrap">
          {formatNumberWithCommas(estimate.finalAmount, displayDecimals)}{" "}
          <span className="text-[10px]">{quoteToken.ticker}</span>
        </span>
      </div>
    </div>
  );
}
