/** Public configuration. `NEXT_PUBLIC_*` values are inlined at build time. */
export const config = {
  apiUrl: process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080",
  registryId: process.env.NEXT_PUBLIC_REGISTRY_CONTRACT_ID ?? "",
  credentialsId: process.env.NEXT_PUBLIC_CREDENTIALS_CONTRACT_ID ?? "",
  rpcUrl: process.env.NEXT_PUBLIC_STELLAR_RPC_URL ?? "https://soroban-testnet.stellar.org",
} as const;

/** The only network this app talks to. Mainnet is not supported. */
export const TESTNET_PASSPHRASE = "Test SDF Network ; September 2015";
