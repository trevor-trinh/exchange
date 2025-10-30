/**
 * Exchange API client singleton
 */

"use client";

import { ExchangeClient } from "@exchange/sdk";

let _exchange: ExchangeClient | null = null;

/**
 * Get or create the singleton ExchangeClient instance.
 * Lazily initializes to avoid SSR issues with environment variables.
 */
export function getExchangeClient(): ExchangeClient {
  if (!_exchange) {
    const apiUrl =
      typeof window !== "undefined"
        ? (window as any).__NEXT_PUBLIC_API_URL__ || process.env.NEXT_PUBLIC_API_URL || "http://localhost:8888"
        : "http://localhost:8888";
    _exchange = new ExchangeClient(apiUrl);
  }
  return _exchange;
}

// For backward compatibility - export as a getter property
export const exchange = new Proxy({} as ExchangeClient, {
  get(_target, prop) {
    return (getExchangeClient() as any)[prop];
  },
});
