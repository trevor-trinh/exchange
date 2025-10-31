export const turnkeyConfig = {
  apiBaseUrl: process.env.NEXT_PUBLIC_TURNKEY_API_BASE_URL || "https://api.turnkey.com",
  defaultOrganizationId: process.env.NEXT_PUBLIC_TURNKEY_ORGANIZATION_ID || "",
  rpId: typeof window !== "undefined" ? window.location.hostname : "localhost",
  iframeUrl: process.env.NEXT_PUBLIC_TURNKEY_IFRAME_URL || "https://auth.turnkey.com",
  serverSignUrl: process.env.NEXT_PUBLIC_API_URL || "http://localhost:8888",
};

// Validate that required environment variables are set
if (!turnkeyConfig.defaultOrganizationId && typeof window !== "undefined") {
  console.warn(
    "NEXT_PUBLIC_TURNKEY_ORGANIZATION_ID is not set. Please set it in .env.local"
  );
}
