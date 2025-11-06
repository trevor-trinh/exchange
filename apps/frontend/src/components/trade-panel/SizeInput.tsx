import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Button } from "@/components/ui/button";
import { toDisplayValue, formatNumber, calculatePercentageSize, roundToLotSize, getDecimalPlaces } from "@exchange/sdk";
import type { Token, Market } from "@/lib/types/exchange";

type OrderSide = "buy" | "sell";

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
    if (!currentPrice) return;

    const result = calculatePercentageSize({
      percentage,
      side,
      availableBase,
      availableQuote,
      currentPrice,
      market,
      baseToken,
    });

    onChange(result);
  };

  return (
    <div className="space-y-1.5">
      <Label className="text-xs font-medium text-muted-foreground">Size ({baseToken.ticker})</Label>
      <Input
        type="number"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onBlur={handleBlur}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.preventDefault();
          }
        }}
        placeholder="0.00"
        step={toDisplayValue(market.lot_size, baseToken.decimals)}
        className="font-mono h-9 text-sm border-border/40 focus:border-primary/50 focus:ring-1 focus:ring-primary/20 bg-muted/20"
      />

      {/* Percentage buttons */}
      <div className="grid grid-cols-4 gap-1.5">
        {[25, 50, 75, 100].map((pct) => (
          <Button
            key={pct}
            type="button"
            variant="outline"
            size="sm"
            onClick={() => setPercentageSize(pct)}
            disabled={!isAuthenticated || !currentPrice}
            className="h-7 text-xs font-medium"
          >
            {pct}%
          </Button>
        ))}
      </div>
    </div>
  );
}
