"use client";

import { useEffect, useRef } from "react";

interface CanvasBackgroundProps {
  dotSize?: number;
  dotColor?: string;
  spacing?: number;
  animationSpeed?: number;
}

export function CanvasBackground({
  dotSize = 2,
  dotColor = "rgba(168, 139, 250, 0.3)", // purple
  spacing = 30,
  animationSpeed = 0.0005,
}: CanvasBackgroundProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    let animationFrameId: number;
    let time = 0;

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };

    resize();
    window.addEventListener("resize", resize);

    const animate = () => {
      if (!ctx || !canvas) return;

      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Draw dot matrix
      for (let x = 0; x < canvas.width; x += spacing) {
        for (let y = 0; y < canvas.height; y += spacing) {
          // Calculate wave effect
          const distance = Math.sqrt(
            Math.pow(x - canvas.width / 2, 2) + Math.pow(y - canvas.height / 2, 2)
          );

          const wave = Math.sin(distance * 0.01 - time) * 0.5 + 0.5;
          const opacity = parseFloat(dotColor.match(/[\d.]+\)$/)?.[0].slice(0, -1) || "0.3");
          const finalOpacity = opacity * wave;

          // Extract RGB values
          const rgbMatch = dotColor.match(/rgba?\((\d+),\s*(\d+),\s*(\d+)/);
          if (rgbMatch) {
            const [, r, g, b] = rgbMatch;
            ctx.fillStyle = `rgba(${r}, ${g}, ${b}, ${finalOpacity})`;

            // Draw dot
            ctx.beginPath();
            ctx.arc(x, y, dotSize, 0, Math.PI * 2);
            ctx.fill();
          }
        }
      }

      time += animationSpeed * 100;
      animationFrameId = requestAnimationFrame(animate);
    };

    animate();

    return () => {
      window.removeEventListener("resize", resize);
      cancelAnimationFrame(animationFrameId);
    };
  }, [dotSize, dotColor, spacing, animationSpeed]);

  return <canvas ref={canvasRef} className="absolute inset-0 pointer-events-none" />;
}
