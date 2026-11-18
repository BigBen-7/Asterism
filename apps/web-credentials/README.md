# web-credentials

Next.js app for the credentials module (Phase 1), testnet only.

- `/` is the **issuer dashboard**: connect the Freighter wallet, register as an issuer, create a schema, authorize yourself under it, issue credentials and revoke them.
- `/verify` is the **public verification page**: it opens a credential link or QR code (the payload is in the URL fragment), or accepts a pasted link or payload, and shows each check and the credential's status.

The app talks to the [credentials API](../../docs/api.md) through `@asterism/sdk`, and to the `registry` and `credentials` contracts through the wallet and the Stellar RPC. Transactions are signed in the wallet; the API never holds keys.

## Configure

Copy the `NEXT_PUBLIC_*` values from `.env.example` into `apps/web-credentials/.env.local`:

| Variable                              | Meaning                                                     |
| ------------------------------------- | ----------------------------------------------------------- |
| `NEXT_PUBLIC_API_URL`                 | Credentials API (default `http://localhost:8080`)           |
| `NEXT_PUBLIC_REGISTRY_CONTRACT_ID`    | Registry contract id on testnet                             |
| `NEXT_PUBLIC_CREDENTIALS_CONTRACT_ID` | Credentials contract id on testnet                          |
| `NEXT_PUBLIC_STELLAR_RPC_URL`         | Soroban RPC (default `https://soroban-testnet.stellar.org`) |

## Run

```bash
pnpm install
pnpm --filter @asterism/web-credentials dev
```

See the [credentials quickstart](../../docs/credentials-quickstart.md) for the full flow.
