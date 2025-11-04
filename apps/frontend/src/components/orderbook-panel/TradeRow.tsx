"use client";

interface TradeRowProps {
  price: string;
  size: string;
  time: string;
  side: "buy" | "sell";
}

export function TradeRow({ price, size, time, side }: TradeRowProps) {
  const colorClass = side === "buy" ? "text-green-500" : "text-red-500";

  return (
    <div className="grid grid-cols-[1fr_0.8fr_1.2fr] items-center text-[11px] leading-tight hover:bg-muted/50 px-3 py-0.5 font-mono tabular-nums">
      <span className={`${colorClass} font-semibold whitespace-nowrap truncate`}>{price}</span>
      <span className="text-muted-foreground text-right whitespace-nowrap truncate">{size}</span>
      <span className="text-muted-foreground text-[10px] text-right whitespace-nowrap truncate">{time}</span>
    </div>
  );
}
