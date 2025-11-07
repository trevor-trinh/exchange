import type { NextConfig } from "next";
import path from "path";

const nextConfig: NextConfig = {
  reactStrictMode: true,
  output: "standalone", // For optimized Docker builds
  // Tell Next.js where the monorepo root is for file tracing
  outputFileTracingRoot: path.join(__dirname, "../.."),
};

export default nextConfig;
