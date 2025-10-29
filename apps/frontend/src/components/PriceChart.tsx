"use client";

import { useEffect, useRef } from "react";
import { usePriceHistory } from "@/lib/hooks";

export function PriceChart() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const { priceHistory, currentPrice } = usePriceHistory();

  useEffect(() => {
    if (!canvasRef.current || priceHistory.length === 0) return;

    const canvas = canvasRef.current;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const { width, height } = canvas;

    // Clear canvas
    ctx.clearRect(0, 0, width, height);

    // Draw background
    ctx.fillStyle = "#000";
    ctx.fillRect(0, 0, width, height);

    // Get price range
    const prices = priceHistory.map((p) => p.price);
    const minPrice = Math.min(...prices);
    const maxPrice = Math.max(...prices);
    const priceRange = maxPrice - minPrice || 1;

    // Draw grid lines
    ctx.strokeStyle = "#333";
    ctx.lineWidth = 1;
    for (let i = 0; i <= 5; i++) {
      const y = (i / 5) * height;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(width, y);
      ctx.stroke();
    }

    // Draw price line
    ctx.strokeStyle = "#3b82f6";
    ctx.lineWidth = 2;
    ctx.beginPath();

    priceHistory.forEach((point, i) => {
      const x = (i / (priceHistory.length - 1)) * width;
      const y = height - ((point.price - minPrice) / priceRange) * (height - 40);

      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }
    });

    ctx.stroke();

    // Draw current price label
    if (currentPrice) {
      ctx.fillStyle = "#fff";
      ctx.font = "bold 20px monospace";
      ctx.fillText(`$${currentPrice.toFixed(2)}`, 20, 40);

      // Draw min/max labels
      ctx.font = "12px monospace";
      ctx.fillStyle = "#888";
      ctx.fillText(`High: $${maxPrice.toFixed(2)}`, 20, height - 30);
      ctx.fillText(`Low: $${minPrice.toFixed(2)}`, 20, height - 10);
    }
  }, [priceHistory, currentPrice]);

  return (
    <div className="p-4 border rounded">
      <h3 className="text-lg font-bold mb-2">Price Chart</h3>
      <canvas
        ref={canvasRef}
        width={800}
        height={400}
        className="w-full"
        style={{ maxWidth: "100%", height: "auto" }}
      />
    </div>
  );
}
