"use client";

import {
  buildVerifyLink,
  createClient,
  issue,
  makeVerifyLink,
  schemaFieldsHash,
  sha256Hex,
  type SchemaDocument,
} from "@asterism/sdk";
import { useState } from "react";
import {
  authorizeIssuerArgs,
  callArgs,
  registerIssuerArgs,
  registerSchemaArgs,
  revokeArgs,
} from "../lib/calls";
import { config } from "../lib/config";
import { invoke } from "../lib/soroban";
import { connectWallet } from "../lib/wallet";

const client = createClient({ apiUrl: config.apiUrl });

const SAMPLE_SCHEMA: SchemaDocument = {
  asterism: "credential-schema/1",
  name: "University degree",
  version: 1,
  fields: {
    degree: { type: "string", required: true },
    graduated: { type: "date", required: true },
  },
};

const SAMPLE_CLAIMS = { degree: "BSc Computer Science", graduated: "2026-07-15" };

const pretty = (value: unknown) => JSON.stringify(value, null, 2);

/** Issuer dashboard: connect a wallet, register as an issuer, create a schema, issue and revoke credentials. */
export function Dashboard() {
  const [address, setAddress] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<{ kind: "ok" | "bad"; text: string } | null>(null);
  const [output, setOutput] = useState<string>("");

  const [issuerName, setIssuerName] = useState("");
  const [schemaText, setSchemaText] = useState(pretty(SAMPLE_SCHEMA));
  const [revocable, setRevocable] = useState(true);
  const [schemaId, setSchemaId] = useState("");
  const [subject, setSubject] = useState("");
  const [claimsText, setClaimsText] = useState(pretty(SAMPLE_CLAIMS));
  const [expiresAt, setExpiresAt] = useState("");
  const [revokeId, setRevokeId] = useState("");

  /** Runs one action, showing progress and any error. */
  async function run(label: string, action: (account: string) => Promise<string | void>) {
    setBusy(true);
    setMessage(null);
    try {
      if (!address) throw new Error("Connect your wallet first.");
      const text = await action(address);
      setMessage({ kind: "ok", text: text ?? `${label}: done` });
    } catch (e) {
      setMessage({
        kind: "bad",
        text: `${label} failed: ${e instanceof Error ? e.message : String(e)}`,
      });
    } finally {
      setBusy(false);
    }
  }

  async function connect() {
    setBusy(true);
    setMessage(null);
    try {
      setAddress(await connectWallet());
    } catch (e) {
      setMessage({ kind: "bad", text: e instanceof Error ? e.message : String(e) });
    } finally {
      setBusy(false);
    }
  }

  const registerIssuer = () =>
    run("Register issuer", async (account) => {
      const name = issuerName.trim();
      if (!name) throw new Error("Enter an issuer name.");
      const metadataHash = await sha256Hex(JSON.stringify({ name }));
      await invoke(
        config.credentialsId,
        "register_issuer",
        registerIssuerArgs(account, name, metadataHash),
        account,
      );
      return `Registered ${account} as "${name}".`;
    });

  const createSchema = () =>
    run("Create schema", async (account) => {
      const document = JSON.parse(schemaText) as SchemaDocument;
      const fieldsHash = await schemaFieldsHash(document);
      const id = await invoke(
        config.registryId,
        "register_schema",
        registerSchemaArgs(account, document.name, document.version, fieldsHash, revocable),
        account,
      );
      setSchemaId(String(id));
      return `Schema registered with id ${String(id)}. Keep the schema document: credentials are checked against it.`;
    });

  const authorizeSelf = () =>
    run("Authorize issuer", async (account) => {
      await invoke(
        config.credentialsId,
        "authorize_issuer",
        authorizeIssuerArgs(account, Number(schemaId), account),
        account,
      );
      return `Authorized ${account} to issue under schema ${schemaId}.`;
    });

  const issueCredential = () =>
    run("Issue credential", async (account) => {
      const schemaDocument = JSON.parse(schemaText) as SchemaDocument;
      const expiresSeconds = expiresAt
        ? Math.floor(new Date(expiresAt).getTime() / 1000)
        : undefined;
      if (expiresSeconds !== undefined && Number.isNaN(expiresSeconds)) {
        throw new Error("Expiry is not a valid date. Use a format like 2027-01-31T00:00:00Z.");
      }
      const prepared = await issue(client, {
        issuer: account,
        schemaId: Number(schemaId),
        schemaDocument,
        subject: subject.trim(),
        claims: JSON.parse(claimsText) as Record<string, unknown>,
        expiresAt: expiresSeconds,
      });
      const attestationId = await invoke(
        prepared.call.contract,
        prepared.call.function,
        callArgs(prepared.call),
        account,
      );

      const payload = {
        attestation_id: Number(attestationId),
        schema_document: schemaDocument,
        credential: prepared.credential,
      };
      let link: string;
      try {
        link = await makeVerifyLink(client, payload);
      } catch {
        link = buildVerifyLink(window.location.origin, { v: 1, ...payload });
      }
      setOutput(
        pretty({ attestation_id: String(attestationId), credential: prepared.credential, link }),
      );
      return `Credential issued as attestation ${String(attestationId)}. Give the holder the credential and link below.`;
    });

  const revoke = () =>
    run("Revoke", async (account) => {
      await invoke(config.credentialsId, "revoke", revokeArgs(account, Number(revokeId)), account);
      return `Attestation ${revokeId} revoked.`;
    });

  return (
    <>
      <h1>Issuer dashboard</h1>
      <p className="muted">Create schemas and issue or revoke credentials on Stellar testnet.</p>

      <div className="card">
        <h2>1. Wallet</h2>
        {address ? (
          <p>
            Connected: <code>{address}</code>
          </p>
        ) : (
          <button disabled={busy} onClick={() => void connect()}>
            Connect Freighter
          </button>
        )}
      </div>

      <div className="card">
        <h2>2. Register as an issuer</h2>
        <label htmlFor="issuer-name">Issuer name</label>
        <input
          id="issuer-name"
          type="text"
          value={issuerName}
          onChange={(e) => setIssuerName(e.target.value)}
        />
        <button disabled={busy || !address} onClick={() => void registerIssuer()}>
          Register issuer
        </button>
      </div>

      <div className="card">
        <h2>3. Create a schema</h2>
        <label htmlFor="schema">Schema document (JSON)</label>
        <textarea id="schema" value={schemaText} onChange={(e) => setSchemaText(e.target.value)} />
        <label>
          <input
            type="checkbox"
            checked={revocable}
            onChange={(e) => setRevocable(e.target.checked)}
          />{" "}
          Credentials under this schema can be revoked
        </label>
        <button disabled={busy || !address} onClick={() => void createSchema()}>
          Create schema
        </button>
        <label htmlFor="schema-id">Schema id</label>
        <input
          id="schema-id"
          type="number"
          min="1"
          value={schemaId}
          onChange={(e) => setSchemaId(e.target.value)}
        />
        <button disabled={busy || !address || !schemaId} onClick={() => void authorizeSelf()}>
          Authorize me to issue under this schema
        </button>
      </div>

      <div className="card">
        <h2>4. Issue a credential</h2>
        <label htmlFor="subject">Holder address</label>
        <input
          id="subject"
          type="text"
          value={subject}
          onChange={(e) => setSubject(e.target.value)}
          placeholder="G..."
        />
        <label htmlFor="claims">Claims (JSON)</label>
        <textarea id="claims" value={claimsText} onChange={(e) => setClaimsText(e.target.value)} />
        <label htmlFor="expires">Expires (optional)</label>
        <input
          id="expires"
          type="text"
          value={expiresAt}
          onChange={(e) => setExpiresAt(e.target.value)}
          placeholder="2027-01-31T00:00:00Z"
        />
        <button
          disabled={busy || !address || !schemaId || !subject}
          onClick={() => void issueCredential()}
        >
          Issue credential
        </button>
      </div>

      <div className="card">
        <h2>5. Revoke a credential</h2>
        <label htmlFor="revoke-id">Attestation id</label>
        <input
          id="revoke-id"
          type="number"
          min="1"
          value={revokeId}
          onChange={(e) => setRevokeId(e.target.value)}
        />
        <button disabled={busy || !address || !revokeId} onClick={() => void revoke()}>
          Revoke
        </button>
      </div>

      {busy && <p className="muted">Working… confirm in your wallet if prompted.</p>}
      {message && (
        <p className={message.kind} role="status">
          {message.text}
        </p>
      )}
      {output && (
        <div className="card">
          <h2>Credential for the holder</h2>
          <p className="muted">Contains the claims and salt. Send it to the holder privately.</p>
          <pre>{output}</pre>
        </div>
      )}
    </>
  );
}
