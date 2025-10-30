"use client";

import { useState } from "react";
import { CreateTokenForm } from "@/components/admin/CreateTokenForm";
import { CreateMarketForm } from "@/components/admin/CreateMarketForm";
import { FaucetForm } from "@/components/admin/FaucetForm";
import { InfoPanel } from "@/components/admin/InfoPanel";
import Link from "next/link";

type TabType = "tokens" | "markets" | "faucet" | "info";

export default function AdminDashboard() {
  const [activeTab, setActiveTab] = useState<TabType>("tokens");

  return (
    <main className="min-h-screen bg-black text-white p-8">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <div className="flex items-center justify-between mb-8">
          <div>
            <h1 className="text-4xl font-bold mb-2">Admin Dashboard</h1>
            <p className="text-gray-400">Manage exchange configuration and operations</p>
          </div>
          <Link href="/" className="px-4 py-2 bg-gray-800 hover:bg-gray-700 rounded-lg transition-colors">
            ← Back to Exchange
          </Link>
        </div>

        {/* Tabs */}
        <div className="flex gap-2 mb-6 border-b border-gray-800">
          <button
            onClick={() => setActiveTab("tokens")}
            className={`px-6 py-3 font-medium transition-colors ${
              activeTab === "tokens" ? "text-white border-b-2 border-blue-500" : "text-gray-400 hover:text-gray-300"
            }`}
          >
            Create Token
          </button>
          <button
            onClick={() => setActiveTab("markets")}
            className={`px-6 py-3 font-medium transition-colors ${
              activeTab === "markets" ? "text-white border-b-2 border-blue-500" : "text-gray-400 hover:text-gray-300"
            }`}
          >
            Create Market
          </button>
          <button
            onClick={() => setActiveTab("faucet")}
            className={`px-6 py-3 font-medium transition-colors ${
              activeTab === "faucet" ? "text-white border-b-2 border-blue-500" : "text-gray-400 hover:text-gray-300"
            }`}
          >
            Faucet
          </button>
          <button
            onClick={() => setActiveTab("info")}
            className={`px-6 py-3 font-medium transition-colors ${
              activeTab === "info" ? "text-white border-b-2 border-blue-500" : "text-gray-400 hover:text-gray-300"
            }`}
          >
            Info
          </button>
        </div>

        {/* Content */}
        <div className="bg-gray-900 rounded-lg p-6">
          {activeTab === "tokens" && <CreateTokenForm />}
          {activeTab === "markets" && <CreateMarketForm />}
          {activeTab === "faucet" && <FaucetForm />}
          {activeTab === "info" && <InfoPanel />}
        </div>
      </div>
    </main>
  );
}
