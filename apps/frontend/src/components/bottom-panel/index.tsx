"use client";

import { Balances } from "./Balances";
import { RecentOrders } from "./RecentOrders";
import { RecentTrades } from "./RecentTrades";
import { Card } from "@/components/ui/card";
import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";

export function BottomPanel() {
  return (
    <div className="w-full h-[600px]">
      <Card className="p-0 overflow-hidden w-full h-full flex flex-col">
        <Tabs defaultValue="balances" className="flex flex-col h-full">
          <TabsList className="justify-start rounded-none border-b border-border h-auto p-0 bg-card backdrop-blur-sm shrink-0">
            <TabsTrigger value="balances" className="rounded-none px-4 text-sm">
              Balances
            </TabsTrigger>
            <TabsTrigger value="orders" className="rounded-none px-4 text-sm ">
              Orders
            </TabsTrigger>
            <TabsTrigger value="trades" className="rounded-none px-4 text-sm ">
              Trades
            </TabsTrigger>
          </TabsList>

          <TabsContent value="balances">
            <Balances />
          </TabsContent>

          <TabsContent value="orders">
            <RecentOrders />
          </TabsContent>

          <TabsContent value="trades">
            <RecentTrades />
          </TabsContent>
        </Tabs>
      </Card>
    </div>
  );
}
