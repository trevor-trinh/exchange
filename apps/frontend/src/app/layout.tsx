import "@turnkey/react-wallet-kit/styles.css";
import "@/styles/globals.css";
import { Geist, Geist_Mono } from "next/font/google";
import Script from "next/script";
import { Providers } from "@/lib/providers";
import { Toaster } from "@/components/ui/sonner";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata = {
  title: "Exchange",
  description: "Trading exchange application",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <Script src="/vendor/trading-view/charting_library.standalone.js" strategy="beforeInteractive" />
      </head>
      <body className={`${geistSans.className} ${geistMono.className} font-sans antialiased`}>
        <Providers>{children}</Providers>
        <Toaster richColors position="bottom-right" />
      </body>
    </html>
  );
}
