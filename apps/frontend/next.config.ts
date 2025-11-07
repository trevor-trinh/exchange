import type { NextConfig } from "next";
import path from "path";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  output: "standalone", // For optimized Docker builds
  experimental: {
    turbopack: {
      // Tell Turbopack the workspace root is two levels up
      root: path.resolve(__dirname, "../.."),
    },
  },
};

export default nextConfig;
