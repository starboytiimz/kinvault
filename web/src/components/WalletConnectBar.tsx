"use client";

import React, { useState } from "react";
import { connectFreighterWallet, WalletSession } from "../lib/wallet";

export function WalletConnectBar() {
  const [session, setSession] = useState<WalletSession | null>(null);
  const [error, setError] = useState<string | null>(null);

  async function handleConnect() {
    setError(null);
    try {
      const s = await connectFreighterWallet("TESTNET");
      setSession(s);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to connect wallet");
    }
  }

  return (
    <div style={{ display: "flex", alignItems: "center", gap: "0.75rem", marginBottom: "1rem" }}>
      <button
        onClick={handleConnect}
        style={{
          padding: "0.5rem 1rem",
          borderRadius: 8,
          border: "1px solid #cbd5e1",
          background: "#f8fafc",
          cursor: "pointer",
        }}
      >
        {session ? `Connected: ${session.publicKey.slice(0, 6)}...` : "Connect Freighter"}
      </button>
      {error && <span style={{ color: "#b91c1c", fontSize: "0.875rem" }}>{error}</span>}
    </div>
  );
}
