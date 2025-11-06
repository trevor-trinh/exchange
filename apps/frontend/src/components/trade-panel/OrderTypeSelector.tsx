import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";

type OrderType = "limit" | "market";

interface OrderTypeSelectorProps {
  value: OrderType;
  onChange: (value: OrderType) => void;
}

export function OrderTypeSelector({ value, onChange }: OrderTypeSelectorProps) {
  return (
    <Tabs value={value} onValueChange={(v) => onChange(v as OrderType)} className="w-full">
      <TabsList className="w-full justify-start rounded-none border-b border-border/40 h-auto p-0 bg-transparent">
        <TabsTrigger value="limit" className="flex-1 rounded-none text-sm">
          Limit
        </TabsTrigger>
        <TabsTrigger value="market" className="flex-1 rounded-none text-sm">
          Market
        </TabsTrigger>
      </TabsList>
    </Tabs>
  );
}
