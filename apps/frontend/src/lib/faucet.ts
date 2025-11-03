import { getExchangeClient } from "./api";
import { toRawValue } from "./format";

/**
 * Auto-faucet: Give new users starting tokens
 */
export async function autoFaucet(userAddress: string, tokens: Array<{ ticker: string; decimals: number }>) {
  const client = getExchangeClient();

  try {
    // Give 10000 of each token to new users
    const faucetPromises = tokens.map(async (token) => {
      const amount = toRawValue(10000, token.decimals);

      try {
        await client.rest.faucet({
          userAddress,
          tokenTicker: token.ticker,
          amount,
          signature: `${userAddress}:${Date.now()}`,
        });
      } catch (err) {
        console.error(`Auto-faucet error for ${token.ticker}:`, err);
      }
    });

    await Promise.all(faucetPromises);
  } catch (err) {
    console.error("Auto-faucet error:", err);
  }
}
