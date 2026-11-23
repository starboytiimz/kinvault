export interface VaultConfigView {
  admin: string;
  token: string;
  frozen: boolean;
  totalDepositedStroops: bigint;
  totalWithdrawnStroops: bigint;
}

export interface MemberAllowanceView {
  member: string;
  label: string;
  capPerPeriodStroops: bigint;
  periodSeconds: number;
  periodStartUnix: number;
  withdrawnInPeriodStroops: bigint;
  bonusBalanceStroops: bigint;
  paused: boolean;
}

export const STROOPS_PER_UNIT = 10_000_000n;

/**
 * Compute remaining withdrawable stroops for a member at `nowUnix`,
 * matching the on-chain rolling period reset logic in `contracts/kinvault`.
 */
export function computeRemainingAllowance(
  member: MemberAllowanceView,
  nowUnix: number
): bigint {
  const periodExpired = nowUnix >= member.periodStartUnix + member.periodSeconds;
  const used = periodExpired ? 0n : member.withdrawnInPeriodStroops;
  const periodRemaining =
    member.capPerPeriodStroops > used ? member.capPerPeriodStroops - used : 0n;
  return periodRemaining + member.bonusBalanceStroops;
}

export function formatUsdc(stroops: bigint): string {
  const whole = stroops / STROOPS_PER_UNIT;
  const frac = (stroops % STROOPS_PER_UNIT).toString().padStart(7, "0").slice(0, 2);
  return `$${whole.toString()}.${frac}`;
}
