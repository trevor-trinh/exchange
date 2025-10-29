/**
 * Exchange API client singleton
 */

"use client";

import { ExchangeClient } from "@exchange/sdk";

// Singleton instance with auto-derived WebSocket URL
export const exchange = new ExchangeClient(process.env.NEXT_PUBLIC_API_URL || "http://localhost:8888");
