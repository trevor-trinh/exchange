"use client";

import { useState } from "react";
import { useExchangeStore, selectSelectedMarket } from "@/lib/store";
import { getExchangeClient } from "@/lib/api";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export function TradePanel() {
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);

  const [side, setSide] = useState<"buy" | "sell">("buy");
  const [orderType, setOrderType] = useState<"limit" | "market">("limit");
  const [price, setPrice] = useState("");
  const [size, setSize] = useState("");
  const [userAddress, setUserAddress] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  if (!selectedMarketId || !selectedMarket) {
    return (
      <Card className="h-full">
        <CardContent className="flex items-center justify-center h-full">
          <p className="text-muted-foreground text-sm">Select a market to trade</p>
        </CardContent>
      </Card>
    );
  }

  const baseToken = tokens.find((t) => t.ticker === selectedMarket.base_ticker);
  const quoteToken = tokens.find((t) => t.ticker === selectedMarket.quote_ticker);

  if (!baseToken || !quoteToken) {
    return (
      <Card className="h-full">
        <CardContent className="flex items-center justify-center h-full">
          <p className="text-muted-foreground text-sm">Loading token information...</p>
        </CardContent>
      </Card>
    );
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(null);
    setLoading(true);

    try {
      if (!userAddress.trim()) {
        throw new Error("User address is required");
      }
      if (!price.trim() && orderType === "limit") {
        throw new Error("Price is required for limit orders");
      }
      if (!size.trim()) {
        throw new Error("Size is required");
      }

      const client = getExchangeClient();

      // For demo purposes, using a simple signature
      // In production, this would be a proper cryptographic signature
      const signature = `${userAddress}:${Date.now()}`;

      const result = await client.placeOrder({
        userAddress: userAddress.trim(),
        marketId: selectedMarketId,
        side: side === "buy" ? "buy" : "sell",
        orderType: orderType === "limit" ? "limit" : "market",
        price: orderType === "limit" ? price : "0",
        size,
        signature,
      });

      setSuccess(`Order placed successfully! Order ID: ${result.order.id.slice(0, 8)}...`);
      setPrice("");
      setSize("");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to place order");
    } finally {
      setLoading(false);
    }
  };

  return (
    <Card className="h-full">
      <CardContent className="p-6">
        {/* Buy/Sell Tabs */}
        <div className="grid grid-cols-2 gap-2 mb-6">
          <Button
            onClick={() => setSide("buy")}
            variant={side === "buy" ? "default" : "outline"}
            className={side === "buy" ? "bg-green-600 hover:bg-green-700 text-white" : ""}
          >
            Buy
          </Button>
          <Button
            onClick={() => setSide("sell")}
            variant={side === "sell" ? "default" : "outline"}
            className={side === "sell" ? "bg-red-600 hover:bg-red-700 text-white" : ""}
          >
            Sell
          </Button>
        </div>

        {/* Order Type */}
        <div className="mb-6">
          <Label className="mb-2 block text-sm font-medium">Order Type</Label>
          <div className="grid grid-cols-2 gap-2">
            <Button
              onClick={() => setOrderType("limit")}
              variant={orderType === "limit" ? "default" : "outline"}
              size="sm"
            >
              Limit
            </Button>
            <Button
              onClick={() => setOrderType("market")}
              variant={orderType === "market" ? "default" : "outline"}
              size="sm"
            >
              Market
            </Button>
          </div>
        </div>

      <form onSubmit={handleSubmit} className="space-y-4">
        {/* User Address */}
        <div className="space-y-2">
          <Label className="text-sm font-medium">User Address</Label>
          <Input
            type="text"
            value={userAddress}
            onChange={(e) => setUserAddress(e.target.value)}
            placeholder="Enter your address"
          />
        </div>

        {/* Price - Only for limit orders */}
        {orderType === "limit" && (
          <div className="space-y-2">
            <Label className="text-sm font-medium">Price ({quoteToken.ticker})</Label>
            <Input
              type="text"
              value={price}
              onChange={(e) => setPrice(e.target.value)}
              placeholder="0.00"
            />
          </div>
        )}

        {/* Size */}
        <div className="space-y-2">
          <Label className="text-sm font-medium">Size ({baseToken.ticker})</Label>
          <Input
            type="text"
            value={size}
            onChange={(e) => setSize(e.target.value)}
            placeholder="0.00"
          />
        </div>

        {/* Total - Only for limit orders */}
        {orderType === "limit" && price && size && (
          <div className="bg-muted border border-border p-3">
            <div className="flex justify-between items-center text-sm">
              <span className="text-muted-foreground">Total</span>
              <span className="text-foreground font-medium">
                {(parseFloat(price) * parseFloat(size)).toFixed(quoteToken.decimals)}{" "}
                {quoteToken.ticker}
              </span>
            </div>
          </div>
        )}

        {/* Error/Success Messages */}
        {error && (
          <div className="bg-red-500/10 border border-red-500/50 p-3 text-red-500 text-sm">
            {error}
          </div>
        )}
        {success && (
          <div className="bg-green-500/10 border border-green-500/50 p-3 text-green-500 text-sm">
            {success}
          </div>
        )}

        {/* Submit Button */}
        <Button
          type="submit"
          disabled={loading}
          className={`w-full ${
            side === "buy"
              ? "bg-green-600 hover:bg-green-700 text-white"
              : "bg-red-600 hover:bg-red-700 text-white"
          }`}
        >
          {loading ? "Placing Order..." : `${side === "buy" ? "Buy" : "Sell"} ${baseToken.ticker}`}
        </Button>
      </form>
      </CardContent>
    </Card>
  );
}
