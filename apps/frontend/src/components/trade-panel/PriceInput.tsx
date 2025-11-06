import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { toDisplayValue, roundToTickSize, getDecimalPlaces } from "@exchange/sdk";
import type { Token, Market } from "@/lib/types/exchange";

interface PriceInputProps {
  value: string;
  onChange: (value: string) => void;
  market: Market;
  quoteToken: Token;
}

export function PriceInput({ value, onChange, market, quoteToken }: PriceInputProps) {
  const priceDecimals = getDecimalPlaces(market.tick_size, quoteToken.decimals);

  const handleBlur = () => {
    if (value) {
      const numPrice = parseFloat(value);
      if (!isNaN(numPrice) && numPrice > 0) {
        const rounded = roundToTickSize(numPrice, market.tick_size, quoteToken.decimals);
        onChange(rounded.toFixed(priceDecimals));
      }
    }
  };

  return (
    <div className="space-y-1.5">
      <Label className="text-xs font-medium text-muted-foreground">Price ({quoteToken.ticker})</Label>
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
        step={toDisplayValue(market.tick_size, quoteToken.decimals)}
        className="font-mono h-9 text-sm border-border/40 focus:border-primary/50 focus:ring-1 focus:ring-primary/20 bg-muted/20"
      />
    </div>
  );
}
