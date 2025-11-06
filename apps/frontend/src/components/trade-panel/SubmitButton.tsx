import { Button } from "@/components/ui/button";
import type { Token } from "@/lib/types/exchange";

type OrderSide = "buy" | "sell";

interface SubmitButtonProps {
  side: OrderSide;
  baseToken: Token;
  isAuthenticated: boolean;
  loading: boolean;
}

export function SubmitButton({ side, baseToken, isAuthenticated, loading }: SubmitButtonProps) {
  const getButtonText = () => {
    if (loading) return "Placing Order...";
    if (!isAuthenticated) return "Connect Wallet";
    return `${side === "buy" ? "Buy" : "Sell"} ${baseToken.ticker}`;
  };

  return (
    <Button
      type="submit"
      disabled={loading || !isAuthenticated}
      size="default"
      className={`w-full font-semibold text-sm h-10 transition-all ${
        side === "buy" ? "bg-green-600 hover:bg-green-700 text-white" : "bg-red-600 hover:bg-red-700 text-white"
      }`}
    >
      {getButtonText()}
    </Button>
  );
}
