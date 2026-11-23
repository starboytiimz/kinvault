"use client";

import React, { useState } from "react";
import {
  computeRemainingAllowance,
  formatUsdc,
  MemberAllowanceView,
} from "../lib/kinvault";

const SAMPLE_MEMBERS: MemberAllowanceView[] = [
  {
    member: "GB5X...TUITION",
    label: "University Tuition & Books",
    capPerPeriodStroops: 150_0000000n,
    periodSeconds: 7 * 86400,
    periodStartUnix: Math.floor(Date.now() / 1000) - 2 * 86400,
    withdrawnInPeriodStroops: 60_0000000n,
    bonusBalanceStroops: 25_0000000n,
    paused: false,
  },
  {
    member: "GD9K...GROCERIES",
    label: "Household Groceries",
    capPerPeriodStroops: 200_0000000n,
    periodSeconds: 7 * 86400,
    periodStartUnix: Math.floor(Date.now() / 1000) - 1 * 86400,
    withdrawnInPeriodStroops: 45_0000000n,
    bonusBalanceStroops: 0n,
    paused: false,
  },
];

export default function KinVaultDashboard() {
  const [frozen, setFrozen] = useState(false);
  const nowUnix = Math.floor(Date.now() / 1000);

  return (
    <main style={{ maxWidth: 840, margin: "2rem auto", fontFamily: "system-ui, sans-serif", padding: "0 1rem" }}>
      <header style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1.5rem" }}>
        <div>
          <h1 style={{ margin: 0 }}>KinVault</h1>
          <p style={{ margin: "0.25rem 0 0", color: "#555" }}>
            Programmable Family Allowance &amp; Household USDC Vault on Soroban
          </p>
        </div>
        <button
          onClick={() => setFrozen((f) => !f)}
          style={{
            padding: "0.5rem 1rem",
            borderRadius: 8,
            border: "none",
            background: frozen ? "#b91c1c" : "#0f172a",
            color: "#fff",
            cursor: "pointer",
          }}
        >
          {frozen ? "Vault Frozen (Unfreeze)" : "Emergency Freeze"}
        </button>
      </header>

      <section style={{ display: "grid", gap: "1rem" }}>
        {SAMPLE_MEMBERS.map((m) => {
          const rem = computeRemainingAllowance(m, nowUnix);
          return (
            <article
              key={m.member}
              style={{ border: "1px solid #e2e8f0", borderRadius: 12, padding: "1rem" }}
            >
              <div style={{ display: "flex", justifyContent: "space-between" }}>
                <strong>{m.label}</strong>
                <span>{formatUsdc(rem)} available</span>
              </div>
              <small style={{ color: "#64748b" }}>
                Member: {m.member} &middot; Weekly Cap: {formatUsdc(m.capPerPeriodStroops)} &middot; Bonus: {formatUsdc(m.bonusBalanceStroops)}
              </small>
            </article>
          );
        })}
      </section>
    </main>
  );
}
