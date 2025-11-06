import { formatNumber } from "@exchange/sdk";
import type { Token } from "@/lib/types/exchange";

type OrderSide = "buy" | "sell";

interface OrderEstimate {
  price: number;
  size: number;
  total: number;
  fee: number;
  finalAmount: number;
}

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
    <div className="relative bg-card border-x border-b border-border/60 rounded-none p-2 pt-3 font-mono text-[10px] leading-tight group cursor-pointer">
      {/* Wavy torn edge at top */}
      <svg
        className="absolute top-0 left-0 w-full h-[10px] overflow-hidden"
        preserveAspectRatio="none"
        viewBox="0 0 100 10"
        xmlns="http://www.w3.org/2000/svg"
      >
        <path
          className="wave-path"
          d="M-12,5 Q-10.5,0 -9,5 T-6,5 T-3,5 T0,5 T3,5 T6,5 T9,5 T12,5 T15,5 T18,5 T21,5 T24,5 T27,5 T30,5 T33,5 T36,5 T39,5 T42,5 T45,5 T48,5 T51,5 T54,5 T57,5 T60,5 T63,5 T66,5 T69,5 T72,5 T75,5 T78,5 T81,5 T84,5 T87,5 T90,5 T93,5 T96,5 T99,5 T102,5 T105,5 T108,5 T111,5 T114,5"
          fill="none"
          stroke="hsl(var(--border))"
          strokeWidth="1.5"
          opacity="0.8"
        />
      </svg>

      {/* Receipt Header */}
      <div className="text-center text-[8px] uppercase tracking-widest text-muted-foreground/60 mb-1.5 mt-1 font-medium">
        Order Summary
      </div>

      {/* Dotted line separator */}
      <div className="border-t border-dashed border-border/50 mb-1.5"></div>

      {/* Order details */}
      <div className="space-y-1">
        <div className="flex justify-between items-center">
          <span className="text-muted-foreground/70 uppercase text-[9px] tracking-wide">Subtotal</span>
          <span className="font-medium tabular-nums text-foreground">
            {formatNumber(estimate.total, displayDecimals)} {quoteToken.ticker}
          </span>
        </div>

        <div className="flex justify-between items-center">
          <span className="text-muted-foreground/70 uppercase text-[9px] tracking-wide">
            Fee ({(Math.abs(feeBps) / 100).toFixed(2)}%)
          </span>
          <span className="font-medium tabular-nums text-foreground">
            {formatNumber(estimate.fee, displayDecimals)} {quoteToken.ticker}
          </span>
        </div>
      </div>

      {/* Dotted line separator */}
      <div className="border-t border-dashed border-border/50 my-1.5"></div>

      {/* Total */}
      <div className="flex justify-between items-center">
        <span className="font-bold uppercase text-[9px] tracking-wider">{side === "buy" ? "You Pay" : "You Get"}</span>
        <span className="font-bold text-[11px] tabular-nums">
          {formatNumber(estimate.finalAmount, displayDecimals)} {quoteToken.ticker}
        </span>
      </div>
    </div>
  );
}
