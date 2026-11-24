export interface WalletSession {
  publicKey: string;
  network: "TESTNET" | "PUBLIC";
  connected: boolean;
}

export const SOROBAN_RPC_URLS = {
  TESTNET: "https://soroban-testnet.stellar.org",
  PUBLIC: "https://mainnet.sorobanrpc.com",
} as const;

/**
 * Connect to Freighter browser wallet when available, returning the active Stellar public key.
 */
export async function connectFreighterWallet(
  network: "TESTNET" | "PUBLIC" = "TESTNET"
): Promise<WalletSession> {
  if (typeof window === "undefined") {
    throw new Error("Wallet connection is only available in a browser environment");
  }
  const freighter = (window as unknown as { freighterApi?: { getPublicKey: () => Promise<string> } })
    .freighterApi;
  if (!freighter) {
    throw new Error("Freighter wallet extension not detected. Install Freighter to sign transactions.");
  }
  const publicKey = await freighter.getPublicKey();
  return { publicKey, network, connected: true };
}

export function getRpcUrl(network: "TESTNET" | "PUBLIC" = "TESTNET"): string {
  return SOROBAN_RPC_URLS[network];
}
