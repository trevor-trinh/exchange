"use client";

import { useEffect, useState } from "react";
import Image from "next/image";
import { CanvasBackground } from "./CanvasBackground";

interface LoadingScreenProps {
  isLoading: boolean;
}

export function LoadingScreen({ isLoading }: LoadingScreenProps) {
  const [isVisible, setIsVisible] = useState(isLoading);
  const [isFadingOut, setIsFadingOut] = useState(false);

  useEffect(() => {
    if (!isLoading && isVisible) {
      // Start fade out animation
      setIsFadingOut(true);
      const timer = setTimeout(() => {
        setIsVisible(false);
      }, 400); // Match the animation duration
      return () => clearTimeout(timer);
    }
  }, [isLoading, isVisible]);

  if (!isVisible) return null;

  return (
    <div
      className={`fixed inset-0 z-[10000] bg-background flex items-center justify-center ${
        isFadingOut ? "loading-exit" : ""
      }`}
      style={{
        imageRendering: "pixelated",
      }}
    >
      {/* Blurred background layer */}
      <div className="absolute inset-0 backdrop-blur-2xl bg-background/90" />

      {/* Animated canvas background */}
      <CanvasBackground dotSize={1.5} spacing={12} animationSpeed={0.003} dotColor="rgba(150, 150, 150, 0.3)" />

      {/* Logo and star */}
      <div className="relative z-10">
        {/* Logo and star horizontal layout */}
        <div className="flex items-center gap-6">
          {/* Logo with pixelated glow animation */}
          <div className="relative group">
            <div className="absolute inset-0 -m-12 bg-white/10 blur-3xl rounded-full animate-pulse" />
            <div className="absolute inset-0 -m-8 bg-primary/20 blur-2xl rounded-full animate-pulse" />
            <div className="relative dither-strong" style={{ imageRendering: "pixelated" }}>
              <Image
                src="/logo3.png"
                alt="Exchange Logo"
                width={160}
                height={160}
                className="h-[160px] w-[160px] contrast-125 brightness-110"
                style={{ imageRendering: "pixelated" }}
                priority
              />
            </div>
          </div>

          {/* Star with multiple effects */}
          <div className="relative">
            {/* Glowing blurred copies behind */}
            <span className="absolute inset-0 text-8xl font-bold text-primary blur-xl animate-pulse opacity-60">*</span>
            <span className="absolute inset-0 text-8xl font-bold text-white blur-lg animate-pulse opacity-40">*</span>
            {/* Main star with dither */}
            <span className="relative text-8xl font-bold text-white animate-pulse inline-block origin-center dither">
              *
            </span>
          </div>
        </div>
      </div>

      {/* Strong dither overlay effect */}
      <div className="dither-strong absolute inset-0 pointer-events-none" />
    </div>
  );
}
