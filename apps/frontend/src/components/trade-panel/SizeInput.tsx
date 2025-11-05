import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { toDisplayValue, roundToLotSize, getDecimalPlaces, formatNumberWithCommas } from "@/lib/format";
import type { Token, Market } from "@/lib/types/openapi";
import type { OrderSide } from "./types";

interface SizeInputProps {
  value: string;
  onChange: (value: string) => void;
  market: Market;
  baseToken: Token;
  quoteToken: Token;
  side: OrderSide;
  availableBase: number;
  availableQuote: number;
  currentPrice: number | null;
  isAuthenticated: boolean;
  error?: string;
}

export function SizeInput({
  value,
  onChange,
  market,
  baseToken,
  quoteToken,
  side,
  availableBase,
  availableQuote,
  currentPrice,
  isAuthenticated,
  error,
}: SizeInputProps) {
  const sizeDecimals = getDecimalPlaces(market.lot_size, baseToken.decimals);

  const handleBlur = () => {
    if (value) {
      const numSize = parseFloat(value);
      if (!isNaN(numSize) && numSize > 0) {
        const rounded = roundToLotSize(numSize, market.lot_size, baseToken.decimals);
        onChange(rounded.toFixed(sizeDecimals));
      }
    }
  };

  const setPercentageSize = (percentage: number) => {
    let maxSize = 0;

    if (side === "buy") {
      // For buy: limited by quote balance / price
      const effectivePrice = currentPrice || 1;
      maxSize = availableQuote / effectivePrice;
    } else {
      // For sell: limited by base balance
      maxSize = availableBase;
    }

    const targetSize = maxSize * (percentage / 100);
    const rounded = roundToLotSize(targetSize, market.lot_size, baseToken.decimals);
    onChange(rounded.toFixed(sizeDecimals));
  };

  return (
    <div className="space-y-2">
      <div className="flex justify-between items-center gap-2">
        <Label className="text-xs font-semibold text-muted-foreground uppercase tracking-wide shrink-0">
          Size ({baseToken.ticker})
        </Label>
        {isAuthenticated && (
          <span
            className="text-xs text-muted-foreground font-medium text-right truncate min-w-0"
            title={`Available: ${formatNumberWithCommas(side === "buy" ? availableQuote : availableBase, 4)} ${side === "buy" ? quoteToken.ticker : baseToken.ticker}`}
          >
            <span className="text-[10px] opacity-70">Available: </span>
            {formatNumberWithCommas(side === "buy" ? availableQuote : availableBase, 2)}{" "}
            <span className="text-[10px]">{side === "buy" ? quoteToken.ticker : baseToken.ticker}</span>
          </span>
        )}
      </div>
      <Input
        type="number"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onBlur={handleBlur}
        placeholder="0.00"
        step={toDisplayValue(market.lot_size, baseToken.decimals)}
        className={`font-mono h-11 text-base border-border/50 focus:border-primary/50 focus:ring-primary/20 bg-muted/30 ${
          error ? "border-red-500/50 focus:border-red-500/50 focus:ring-red-500/20" : ""
        }`}
      />
      {error && <p className="text-xs text-red-600 font-medium">{error}</p>}

      {/* Percentage buttons */}
      <div className="grid grid-cols-4 gap-2">
        {[25, 50, 75, 100].map((pct) => (
          <Button
            key={pct}
            type="button"
            variant="outline"
            size="sm"
            onClick={() => setPercentageSize(pct)}
            disabled={!isAuthenticated}
            className="h-8 text-xs font-semibold transition-all"
          >
            {pct}%
          </Button>
        ))}
      </div>
    </div>
  );
}
