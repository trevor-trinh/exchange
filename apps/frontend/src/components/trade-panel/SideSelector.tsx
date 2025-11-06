import { Button } from "@/components/ui/button";

type OrderSide = "buy" | "sell";

interface SideSelectorProps {
  value: OrderSide;
  onChange: (value: OrderSide) => void;
}

export function SideSelector({ value, onChange }: SideSelectorProps) {
  return (
    <div className="grid grid-cols-2 gap-2">
      <Button
        type="button"
        onClick={() => onChange("buy")}
        variant={value === "buy" ? "default" : "outline"}
        className={
          value === "buy"
            ? "bg-green-600 hover:bg-green-700 text-white border-green-500/30"
            : "hover:bg-green-600/5 border-green-600/20 hover:border-green-600/30"
        }
        size="default"
      >
        Buy
      </Button>
      <Button
        type="button"
        onClick={() => onChange("sell")}
        variant={value === "sell" ? "default" : "outline"}
        className={
          value === "sell"
            ? "bg-red-600 hover:bg-red-700 text-white border-red-500/30"
            : "hover:bg-red-600/5 border-red-600/20 hover:border-red-600/30"
        }
        size="default"
      >
        Sell
      </Button>
    </div>
  );
}
