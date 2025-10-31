"use client";

import { TurnkeyProvider as BaseTurnkeyProvider } from "@turnkey/sdk-react";
import { turnkeyConfig } from "./config";

export function TurnkeyProvider({ children }: { children: React.ReactNode }) {
  return (
    <BaseTurnkeyProvider config={turnkeyConfig}>
      {children}
    </BaseTurnkeyProvider>
  );
}
