import "@/styles/globals.css";
import { Geist, Geist_Mono } from "next/font/google";
import Script from "next/script";

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
    <html lang="en">
      <head>
        <Script src="/vendor/trading-view/charting_library.standalone.js" strategy="beforeInteractive" />
      </head>
      <body className={`${geistSans.className} ${geistMono.className} font-sans antialiased`}>{children}</body>
    </html>
  );
}
