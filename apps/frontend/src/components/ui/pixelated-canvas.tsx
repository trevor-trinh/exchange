"use client";

import React, { useRef, useEffect, useState } from "react";
import { motion } from "framer-motion";

interface PixelatedCanvasProps {
  src: string;
  width?: number;
  height?: number;
  cellSize?: number;
  dotScale?: number;
  shape?: "circle" | "square";
  interactive?: boolean;
  distortionStrength?: number;
  distortionRadius?: number;
  distortionMode?: "repel" | "attract" | "swirl";
  className?: string;
}

interface Dot {
  x: number;
  y: number;
  baseX: number;
  baseY: number;
  color: string;
  size: number;
}

export function PixelatedCanvas({
  src,
  width = 400,
  height = 500,
  cellSize = 3,
  dotScale = 0.9,
  shape = "square",
  interactive = true,
  distortionStrength = 3,
  distortionRadius = 80,
  distortionMode = "swirl",
  className = "",
}: PixelatedCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [dots, setDots] = useState<Dot[]>([]);
  const [imageLoaded, setImageLoaded] = useState(false);
  const mousePos = useRef({ x: -1000, y: -1000 });
  const animationFrameId = useRef<number>();

  // Load and process image
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const img = new Image();
    img.crossOrigin = "anonymous";

    img.onload = () => {
      // Set canvas size to match image
      canvas.width = width;
      canvas.height = height;

      // Draw image
      ctx.drawImage(img, 0, 0, width, height);

      // Extract pixel data
      const imageData = ctx.getImageData(0, 0, width, height);
      const pixels = imageData.data;

      // Create dots
      const newDots: Dot[] = [];
      for (let y = 0; y < height; y += cellSize) {
        for (let x = 0; x < width; x += cellSize) {
          const i = (y * width + x) * 4;
          const r = pixels[i];
          const g = pixels[i + 1];
          const b = pixels[i + 2];
          const a = pixels[i + 3];

          // Skip transparent pixels
          if (a < 10) continue;

          newDots.push({
            x,
            y,
            baseX: x,
            baseY: y,
            color: `rgba(${r}, ${g}, ${b}, ${a / 255})`,
            size: cellSize * dotScale,
          });
        }
      }

      setDots(newDots);
      setImageLoaded(true);
    };

    img.src = src;
  }, [src, width, height, cellSize, dotScale]);

  // Animation loop
  useEffect(() => {
    if (!imageLoaded || !interactive) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const animate = () => {
      ctx.clearRect(0, 0, width, height);

      dots.forEach((dot) => {
        let x = dot.baseX;
        let y = dot.baseY;

        if (interactive) {
          const dx = mousePos.current.x - dot.baseX;
          const dy = mousePos.current.y - dot.baseY;
          const distance = Math.sqrt(dx * dx + dy * dy);

          if (distance < distortionRadius) {
            const force = (distortionRadius - distance) / distortionRadius;

            if (distortionMode === "repel") {
              x -= (dx / distance) * force * distortionStrength;
              y -= (dy / distance) * force * distortionStrength;
            } else if (distortionMode === "attract") {
              x += (dx / distance) * force * distortionStrength;
              y += (dy / distance) * force * distortionStrength;
            } else if (distortionMode === "swirl") {
              const angle = Math.atan2(dy, dx) + force * Math.PI * 0.5;
              const dist = force * distortionStrength;
              x += Math.cos(angle) * dist;
              y += Math.sin(angle) * dist;
            }
          }
        }

        ctx.fillStyle = dot.color;

        if (shape === "circle") {
          ctx.beginPath();
          ctx.arc(x, y, dot.size / 2, 0, Math.PI * 2);
          ctx.fill();
        } else {
          ctx.fillRect(x - dot.size / 2, y - dot.size / 2, dot.size, dot.size);
        }
      });

      animationFrameId.current = requestAnimationFrame(animate);
    };

    animate();

    return () => {
      if (animationFrameId.current) {
        cancelAnimationFrame(animationFrameId.current);
      }
    };
  }, [dots, imageLoaded, interactive, distortionStrength, distortionRadius, distortionMode, shape, width, height]);

  // Mouse tracking
  useEffect(() => {
    if (!interactive) return;

    const canvas = canvasRef.current;
    if (!canvas) return;

    const handleMouseMove = (e: MouseEvent) => {
      const rect = canvas.getBoundingClientRect();
      mousePos.current = {
        x: (e.clientX - rect.left) * (width / rect.width),
        y: (e.clientY - rect.top) * (height / rect.height),
      };
    };

    const handleMouseLeave = () => {
      mousePos.current = { x: -1000, y: -1000 };
    };

    canvas.addEventListener("mousemove", handleMouseMove);
    canvas.addEventListener("mouseleave", handleMouseLeave);

    return () => {
      canvas.removeEventListener("mousemove", handleMouseMove);
      canvas.removeEventListener("mouseleave", handleMouseLeave);
    };
  }, [interactive, width, height]);

  return (
    <motion.canvas
      ref={canvasRef}
      className={className}
      style={{ width, height }}
      initial={{ opacity: 0 }}
      animate={{ opacity: imageLoaded ? 1 : 0 }}
      transition={{ duration: 0.5 }}
    />
  );
}
