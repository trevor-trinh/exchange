"use client";

import { useEffect, useMemo, useState } from "react";
import { useForm } from "react-hook-form";
import { useExchangeStore, selectSelectedMarket, selectOrderbookBids, selectOrderbookAsks } from "@/lib/store";
import { useUserBalances } from "@/lib/hooks";
import { useExchangeClient } from "@/lib/hooks/useExchangeClient";
import { Card, CardContent } from "@/components/ui/card";
import { toDisplayValue, formatNumber, roundToTickSize, getDecimalPlaces } from "@exchange/sdk";
import { OrderTypeSelector } from "./OrderTypeSelector";
import { SideSelector } from "./SideSelector";
import { PriceInput } from "./PriceInput";
import { SizeInput } from "./SizeInput";
import { OrderSummary } from "./OrderSummary";
import { SubmitButton } from "./SubmitButton";
import { FaucetDialog } from "@/components/FaucetDialog";

type OrderSide = "buy" | "sell";
type OrderType = "limit" | "market";

interface TradeFormData {
  side: OrderSide;
  orderType: OrderType;
  price: string;
  size: string;
}

export function TradePanel() {
  const client = useExchangeClient();
  const selectedMarketId = useExchangeStore((state) => state.selectedMarketId);
  const selectedMarket = useExchangeStore(selectSelectedMarket);
  const tokens = useExchangeStore((state) => state.tokens);
  const userAddress = useExchangeStore((state) => state.userAddress);
  const isAuthenticated = useExchangeStore((state) => state.isAuthenticated);
  const recentTrades = useExchangeStore((state) => state.recentTrades);
  const bids = useExchangeStore(selectOrderbookBids);
  const asks = useExchangeStore(selectOrderbookAsks);
  const selectedPrice = useExchangeStore((state) => state.selectedPrice);
  const setSelectedPrice = useExchangeStore((state) => state.setSelectedPrice);
  const balances = useUserBalances();

  const [loading, setLoading] = useState(false);
  const [success, setSuccess] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [faucetOpen, setFaucetOpen] = useState(false);

  // React Hook Form
  const {
    watch,
    setValue,
    handleSubmit: rhfHandleSubmit,
  } = useForm<TradeFormData>({
    defaultValues: {
      side: "buy",
      orderType: "limit",
      price: "",
      size: "",
    },
  });

  const formData = watch();

  // Look up tokens for the selected market
  const baseToken = selectedMarket ? tokens[selectedMarket.base_ticker] : undefined;
  const quoteToken = selectedMarket ? tokens[selectedMarket.quote_ticker] : undefined;

  // Get user balances
  const baseBalance = balances.find((b) => b.token_ticker === baseToken?.ticker);
  const quoteBalance = balances.find((b) => b.token_ticker === quoteToken?.ticker);

  // Calculate available balances
  const availableBase = baseBalance ? baseBalance.amountValue - baseBalance.lockedValue : 0;
  const availableQuote = quoteBalance ? quoteBalance.amountValue - quoteBalance.lockedValue : 0;

  // Get price helpers
  const lastTradePrice = recentTrades.length > 0 && recentTrades[0] ? recentTrades[0].priceValue : null;
  const bestBid = bids.length > 0 && bids[0] ? bids[0].priceValue : null;
  const bestAsk = asks.length > 0 && asks[0] ? asks[0].priceValue : null;

  // Calculate current price for size calculations
  const currentPrice =
    formData.orderType === "limit"
      ? parseFloat(formData.price) || null
      : (formData.side === "buy" ? bestAsk : bestBid) || lastTradePrice;

  // Calculate decimal places
  const priceDecimals =
    selectedMarket && quoteToken ? getDecimalPlaces(selectedMarket.tick_size, quoteToken.decimals) : 2;
  const sizeDecimals = selectedMarket && baseToken ? getDecimalPlaces(selectedMarket.lot_size, baseToken.decimals) : 2;

  // Calculate order estimate inline
  const estimate = useMemo(() => {
    if (!selectedMarket) return null;

    const priceNum = parseFloat(formData.price);
    const sizeNum = parseFloat(formData.size);

    let effectivePrice = 0;
    if (formData.orderType === "limit") {
      effectivePrice = priceNum;
    } else {
      effectivePrice = (formData.side === "buy" ? bestAsk : bestBid) || lastTradePrice || 0;
    }

    if (isNaN(effectivePrice) || effectivePrice <= 0 || isNaN(sizeNum) || sizeNum <= 0) {
      return null;
    }

    const total = effectivePrice * sizeNum;
    const feeBps = formData.orderType === "market" ? selectedMarket.taker_fee_bps : selectedMarket.maker_fee_bps;
    const fee = (total * Math.abs(feeBps)) / 10000;
    const finalAmount = formData.side === "buy" ? total + fee : total - fee;

    return {
      price: effectivePrice,
      size: sizeNum,
      total,
      fee,
      finalAmount,
    };
  }, [formData, selectedMarket, bestBid, bestAsk, lastTradePrice]);

  // Handle price selection from orderbook
  useEffect(() => {
    if (selectedPrice !== null && selectedMarket && baseToken && quoteToken) {
      // Auto-switch to limit order if currently on market order
      if (formData.orderType === "market") {
        setValue("orderType", "limit");
      }

      // Round price to tick size and set it
      const rounded = roundToTickSize(selectedPrice, selectedMarket.tick_size, quoteToken.decimals);
      setValue("price", rounded.toFixed(priceDecimals));

      // Clear the selected price from store
      setSelectedPrice(null);
    }
  }, [
    selectedPrice,
    selectedMarket,
    baseToken,
    quoteToken,
    formData.orderType,
    setSelectedPrice,
    setValue,
    priceDecimals,
  ]);

  // Form submission
  const onSubmit = async (data: TradeFormData) => {
    setError(null);
    setSuccess(null);

    // Check market data
    if (!selectedMarket || !baseToken || !quoteToken) {
      setError("Market data not loaded");
      return;
    }

    // Check authentication
    if (!isAuthenticated || !userAddress) {
      setError("Please connect your wallet first");
      return;
    }

    // Simple validation
    if (data.orderType === "limit" && (!data.price.trim() || parseFloat(data.price) <= 0)) {
      setError("Invalid price");
      return;
    }

    if (!data.size.trim() || parseFloat(data.size) <= 0) {
      setError("Invalid size");
      return;
    }

    // Check balance
    const sizeNum = parseFloat(data.size);
    if (data.side === "buy") {
      const priceNum = data.orderType === "limit" ? parseFloat(data.price) : bestAsk || lastTradePrice || 0;
      const requiredQuote = sizeNum * priceNum;
      if (requiredQuote > availableQuote) {
        setError(`Insufficient ${quoteToken.ticker} balance`);
        return;
      }
    } else {
      if (sizeNum > availableBase) {
        setError(`Insufficient ${baseToken.ticker} balance`);
        return;
      }
    }

    setLoading(true);

    try {
      const finalPrice = data.orderType === "limit" ? parseFloat(data.price) : 0;
      const finalSize = parseFloat(data.size);

      // For demo purposes, using a simple signature
      const signature = `${userAddress}:${Date.now()}`;

      // Use SDK's placeOrderDecimal - handles conversion and rounding
      const result = await client.rest.placeOrderDecimal({
        userAddress,
        marketId: selectedMarket.id,
        side: data.side,
        orderType: data.orderType,
        priceDecimal: finalPrice.toString(),
        sizeDecimal: finalSize.toString(),
        signature,
      });

      const successMessage = `Order placed! ${
        result.trades.length > 0 ? `Filled ${result.trades.length} trade(s)` : "Order in book"
      }`;
      setSuccess(successMessage);

      // Clear form
      setValue("price", "");
      setValue("size", "");

      // Auto-clear success message after 3 seconds
      setTimeout(() => setSuccess(null), 3000);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : "Failed to place order";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  // Early returns for loading states
  if (!selectedMarketId || !selectedMarket) {
    return (
      <Card className="h-full min-h-[400px]">
        <CardContent className="flex items-center justify-center h-full">
          <p className="text-muted-foreground text-sm">Select a market to trade</p>
        </CardContent>
      </Card>
    );
  }

  if (!baseToken || !quoteToken) {
    return (
      <Card className="h-full min-h-[400px]">
        <CardContent className="flex items-center justify-center h-full">
          <p className="text-muted-foreground text-sm">Loading token information...</p>
        </CardContent>
      </Card>
    );
  }

  // Calculate fee bps based on order type
  const feeBps = formData.orderType === "market" ? selectedMarket.taker_fee_bps : selectedMarket.maker_fee_bps;

  return (
    <Card className="h-full flex flex-col gap-0 py-0 overflow-hidden border-border/40 bg-card min-w-0">
      <OrderTypeSelector value={formData.orderType} onChange={(value) => setValue("orderType", value)} />

      <form onSubmit={rhfHandleSubmit(onSubmit)} className="flex-1 flex flex-col min-h-0">
        <CardContent className="p-3 space-y-3 flex-1 overflow-y-auto">
          {/* Buy/Sell Buttons */}
          <SideSelector value={formData.side} onChange={(value) => setValue("side", value)} />

          {/* Available Balance */}
          <button
            type="button"
            onClick={() => isAuthenticated && setFaucetOpen(true)}
            className="under text-[10px] text-muted-foreground/60 hover:text-primary/70 transition-colors cursor-pointer w-full -mt-1 flex justify-between items-center py-1"
            disabled={!isAuthenticated}
          >
            <span className="opacity-70 underline-offset-2 underline decoration-dotted">Available:</span>
            <span className="font-medium">
              {isAuthenticated ? formatNumber(formData.side === "buy" ? availableQuote : availableBase, 4) : "0.00"}{" "}
              {formData.side === "buy" ? quoteToken?.ticker : baseToken?.ticker}
            </span>
          </button>

          {/* Price - Only for limit orders */}
          {formData.orderType === "limit" && (
            <PriceInput
              value={formData.price}
              onChange={(value) => setValue("price", value)}
              market={selectedMarket}
              quoteToken={quoteToken}
            />
          )}

          {/* Size */}
          <SizeInput
            value={formData.size}
            onChange={(value) => setValue("size", value)}
            market={selectedMarket}
            baseToken={baseToken}
            quoteToken={quoteToken}
            side={formData.side}
            availableBase={availableBase}
            availableQuote={availableQuote}
            currentPrice={currentPrice}
            isAuthenticated={isAuthenticated}
          />

          {/* Error/Success Messages */}
          {error && (
            <div className="bg-red-500/10 border border-red-500/30 rounded-md p-2 text-red-600 text-xs font-medium">
              {error}
            </div>
          )}
          {success && (
            <div className="bg-green-500/10 border border-green-500/30 rounded-md p-2 text-green-600 text-xs font-medium">
              {success}
            </div>
          )}
        </CardContent>

        {/* Bottom section with summary and button */}
        <div className="p-3 space-y-3 mt-auto">
          {/* Estimated total and fees */}
          <OrderSummary
            estimate={estimate}
            side={formData.side}
            quoteToken={quoteToken}
            priceDecimals={priceDecimals}
            feeBps={feeBps}
          />

          {/* Submit Button */}
          <SubmitButton
            side={formData.side}
            baseToken={baseToken}
            isAuthenticated={isAuthenticated}
            loading={loading}
          />
        </div>
      </form>

      {/* Faucet Dialog - controlled by available balance click */}
      <FaucetDialog controlled open={faucetOpen} onOpenChange={setFaucetOpen} />
    </Card>
  );
}
