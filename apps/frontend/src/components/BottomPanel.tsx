"use client";

import { useState } from "react";
import { Balances } from "./Balances";
import { RecentOrders } from "./RecentOrders";

type TabType = "balances" | "orders";

export function BottomPanel() {
  const [activeTab, setActiveTab] = useState<TabType>("balances");

  return (
    <div className="bg-[#0f0f0f] rounded-xl border border-gray-800/30 backdrop-blur-sm flex flex-col overflow-hidden">
      {/* Tabs */}
      <div className="flex border-b border-gray-800/30">
        <button
          onClick={() => setActiveTab("balances")}
          className={`px-6 py-3 text-xs font-semibold uppercase tracking-wider transition-all ${
            activeTab === "balances"
              ? "text-white bg-gradient-to-b from-blue-500/10 to-transparent border-b-2 border-blue-500"
              : "text-gray-500 hover:text-gray-300 hover:bg-white/5"
          }`}
        >
          Balances
        </button>
        <button
          onClick={() => setActiveTab("orders")}
          className={`px-6 py-3 text-xs font-semibold uppercase tracking-wider transition-all ${
            activeTab === "orders"
              ? "text-white bg-gradient-to-b from-blue-500/10 to-transparent border-b-2 border-blue-500"
              : "text-gray-500 hover:text-gray-300 hover:bg-white/5"
          }`}
        >
          Orders
        </button>
      </div>

      {/* Content */}
      <div className="p-5">
        {activeTab === "balances" ? <Balances /> : <RecentOrders />}
      </div>
    </div>
  );
}
