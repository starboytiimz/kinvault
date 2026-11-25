"use client";

import React, { useState } from "react";
import { computeRemainingAllowance, formatUsdc, MemberAllowanceView } from "../lib/kinvault";

export function MemberWithdrawCard({ member }: { member: MemberAllowanceView }) {
  const [amountInput, setAmountInput] = useState("25");
  const [notice, setNotice] = useState<string | null>(null);
  const available = computeRemainingAllowance(member, Math.floor(Date.now() / 1000));

  function handlePreview() {
    const stroops = BigInt(Math.round(Number(amountInput) * 1e7));
    if (stroops <= 0n) {
      setNotice("Enter a positive USDC amount.");
    } else if (stroops > available) {
      setNotice(`Exceeds available allowance (${formatUsdc(available)}).`);
    } else {
      setNotice(`Ready to submit Soroban withdraw(${formatUsdc(stroops)}).`);
    }
  }

  return (
    <div style={{ border: "1px solid #e2e8f0", borderRadius: 10, padding: "1rem" }}>
      <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "0.5rem" }}>
        <strong>{member.label}</strong>
        <span>{formatUsdc(available)} available</span>
      </div>
      <div style={{ display: "flex", gap: "0.5rem" }}>
        <input
          type="number"
          value={amountInput}
          onChange={(e) => setAmountInput(e.target.value)}
          style={{ padding: "0.4rem", borderRadius: 6, border: "1px solid #cbd5e1", width: 100 }}
        />
        <button onClick={handlePreview} style={{ padding: "0.4rem 0.9rem", borderRadius: 6, cursor: "pointer" }}>
          Preview Withdraw
        </button>
      </div>
      {notice && <p style={{ margin: "0.5rem 0 0", fontSize: "0.85rem", color: "#334155" }}>{notice}</p>}
    </div>
  );
}
