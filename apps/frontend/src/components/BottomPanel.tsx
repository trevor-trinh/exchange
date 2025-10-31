"use client";

import { Balances } from "./Balances";
import { RecentOrders } from "./RecentOrders";
import { RecentTrades } from "./RecentTrades";
import { Card } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";

export function BottomPanel() {
  return (
    <Card className="py-0 overflow-hidden">
      <Tabs defaultValue="balances">
        <TabsList className="w-full justify-start rounded-none border-b border-border h-auto p-0 bg-card/50 backdrop-blur-sm sticky top-0 z-10">
          <TabsTrigger value="balances" className="rounded-none px-4">
            Balances
          </TabsTrigger>
          <TabsTrigger value="orders" className="rounded-none px-4">
            Orders
          </TabsTrigger>
          <TabsTrigger value="trades" className="rounded-none px-4">
            Trades
          </TabsTrigger>
        </TabsList>

        <TabsContent value="balances" className="px-6 pb-6 pt-4">
          <Balances />
        </TabsContent>

        <TabsContent value="orders" className="px-6 pb-6 pt-4">
          <RecentOrders />
        </TabsContent>

        <TabsContent value="trades" className="px-6 pb-6 pt-4">
          <RecentTrades />
        </TabsContent>
      </Tabs>
    </Card>
  );
}
