# Credentials quickstart

Issue a credential on Stellar testnet and verify it, using the web app or the SDK. Testnet only; unaudited software.

## What you are building

1. A **schema** says which claims a credential has (RFC 0003). Its hash is registered on-chain in the registry.
2. An **issuer** registers on the `credentials` contract, and the schema's creator authorizes it.
3. The issuer **issues** a credential: the [API](api.md) validates the claims and prepares the commitment and contract call; the issuer signs and submits it. Only a hash goes on-chain; the claims stay with the holder.
4. A **verifier** opens the holder's link (or QR code). The API checks the claims against the on-chain commitment and reports whether the credential is valid, expired or revoked.

## Prerequisites

- The registry and `credentials` contracts deployed on testnet. [Deploy the registry](deploy-testnet.md); the `credentials` contract is deployed the same way (`stellar contract deploy` with its wasm and the registry's contract id as the constructor argument).
- The [indexer](../services/indexer/README.md) running against a local Postgres, started at or before the registry's deployment ledger.
- The [API](api.md) running with `REGISTRY_CONTRACT_ID` and `CREDENTIALS_CONTRACT_ID` set.
- For the web app: the [Freighter](https://www.freighter.app) wallet set to the **Test network**, with a funded testnet account. A funded account can be created with `stellar keys generate <name> --network testnet --fund`.

See [development.md](development.md) for building the repository.

## With the web app

Start `apps/web-credentials` (see its [README](../apps/web-credentials/README.md)) and open it.

1. **Connect** Freighter.
2. **Register as an issuer** with a name.
3. **Create a schema** from a schema document (a sample is prefilled). The app registers its hash on-chain and shows the schema id.
4. **Authorize** yourself to issue under that schema.
5. **Issue a credential** to a holder address with its claims. Confirm the transaction in your wallet. The app shows the credential record and a verification link. Give both to the holder; the record contains the claims and salt, so send it privately.
6. **Verify**: open the link, which goes to `/verify`. It shows each check and the status.
7. **Revoke** with the attestation id if needed. Revoking needs a schema registered as revocable, and only the issuer can do it.

## With the SDK

The SDK is the workspace package `@asterism/sdk`. It does not sign or submit transactions: it prepares them through the API. Signing is yours (a wallet, or the Stellar CLI or SDK in a backend).

```ts
import { buildVerifyLink, createClient, issue, schemaFieldsHash, verify } from "@asterism/sdk";
import type { SchemaDocument } from "@asterism/sdk";

const client = createClient({ apiUrl: "http://localhost:8080" });

const schemaDocument: SchemaDocument = {
  asterism: "credential-schema/1",
  name: "University degree",
  version: 1,
  fields: {
    degree: { type: "string", required: true },
    graduated: { type: "date", required: true },
  },
};

// 1. The hash to pass as `fields_hash` to the registry's `register_schema`.
const fieldsHash = await schemaFieldsHash(schemaDocument);

// 2. Prepare a credential. `call` is the `credentials.issue` call to sign and submit.
const prepared = await issue(client, {
  issuer: "G...issuer",
  schemaId: 1, // the id returned by register_schema
  schemaDocument,
  subject: "G...holder",
  claims: { degree: "BSc Computer Science", graduated: "2026-07-15" },
});
// prepared.call = { contract, function: "issue", args: { issuer, schema_id, subject, data_hash, expires_at } }
// Sign and submit it; the contract returns the attestation id.

// 3. After submitting, build the link for the holder.
const attestationId = 12; // returned by the issue call
const link = buildVerifyLink("https://verify.example", {
  v: 1,
  attestation_id: attestationId,
  schema_document: schemaDocument,
  credential: prepared.credential,
});

// 4. Anyone can verify with the link (or with `{ payload }`).
const result = await verify(client, { link });
console.log(result.valid, result.status); // true "valid"
```

`verify` resolves with `valid: false` and the failed checks for a credential that does not verify, and throws an `ApiError` for a malformed request. For many credentials at once use `issueBatch`, which returns one Merkle root and a proof per credential; add the proof to each holder's payload (`proof`).

Submitting the `issue` call from a Node script or backend is the same as for any Soroban contract call. For example with the Stellar CLI, using the values from `prepared.call.args`:

```bash
stellar contract invoke --id <credentials contract id> --source-account <issuer identity> --network testnet \
  -- issue --issuer <issuer> --schema_id 1 --subject <holder> --data_hash <data_hash> --expires_at null
```

The exact argument syntax for `Option` values may differ between CLI versions (`needs-verification`); check `stellar contract invoke ... -- issue --help`.

## Troubleshooting

| Symptom                                | Likely cause                                                                     |
| -------------------------------------- | -------------------------------------------------------------------------------- |
| `IssuerNotAuthorized` on issue         | The issuer is not registered, or the schema creator has not authorized it.       |
| `attestation_found` check fails        | The indexer has not caught up yet, or is not running against this registry.      |
| `schema_document_hash` check fails     | The schema document differs from the one registered (any edit changes the hash). |
| `commitment` check fails               | The claims, salt or subject were altered, or the wrong attestation id is used.   |
| Freighter asks for a different network | Switch Freighter to the Test network. Mainnet is not supported.                  |
