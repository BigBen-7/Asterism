"use client";

import {
  ApiError,
  createClient,
  parseVerifyLink,
  verify,
  type VerifyPayload,
  type VerifyResult,
} from "@asterism/sdk";
import { useEffect, useState } from "react";
import { config } from "../lib/config";

const client = createClient({ apiUrl: config.apiUrl });

type State =
  | { kind: "idle" }
  | { kind: "loading" }
  | { kind: "done"; result: VerifyResult; subject?: string; claims?: Record<string, unknown> }
  | { kind: "error"; message: string };

const STATUS_CLASS: Record<string, string> = { valid: "ok", expired: "warn", revoked: "bad" };

/** Public verification page: reads a credential link (or pasted payload) and checks it against the chain. */
export function VerifyView() {
  const [input, setInput] = useState("");
  const [state, setState] = useState<State>({ kind: "idle" });

  async function run(text: string) {
    setState({ kind: "loading" });
    try {
      const trimmed = text.trim();
      if (!trimmed) throw new Error("Paste a credential link or payload first.");
      const isLink = !trimmed.startsWith("{");
      const payload: VerifyPayload = isLink
        ? parseVerifyLink(trimmed)
        : (JSON.parse(trimmed) as VerifyPayload);
      const result = await verify(client, { payload });
      setState({
        kind: "done",
        result,
        subject: payload.credential.subject,
        claims: payload.credential.claims,
      });
    } catch (e) {
      setState({
        kind: "error",
        message: e instanceof ApiError || e instanceof Error ? e.message : "Verification failed",
      });
    }
  }

  // A scanned QR code or opened link carries the payload in the URL fragment.
  useEffect(() => {
    if (window.location.hash.length > 1) {
      const link = window.location.href;
      setInput(link);
      void run(link);
    }
    // Run once on load.
  }, []);

  return (
    <>
      <h1>Verify a credential</h1>
      <p className="muted">
        Open a credential link or QR code, or paste a link or payload below. The check runs against
        Stellar testnet.
      </p>

      <div className="card">
        <label htmlFor="credential">Credential link or payload (JSON)</label>
        <textarea id="credential" value={input} onChange={(e) => setInput(e.target.value)} />
        <button disabled={state.kind === "loading"} onClick={() => void run(input)}>
          {state.kind === "loading" ? "Verifying…" : "Verify"}
        </button>
      </div>

      {state.kind === "error" && <p className="bad">{state.message}</p>}

      {state.kind === "done" && (
        <div className="card" aria-live="polite">
          <h2 className={state.result.valid ? "ok" : "bad"}>
            {state.result.valid ? "Valid credential" : "Not valid"}
            {state.result.status && (
              <span className={STATUS_CLASS[state.result.status]}> ({state.result.status})</span>
            )}
          </h2>
          <ul className="checks">
            {state.result.checks.map((check) => (
              <li key={check.name} className={check.passed ? "ok" : "bad"}>
                {check.passed ? "✓" : "✗"} {check.name}
                {check.detail && <span className="muted"> — {check.detail}</span>}
              </li>
            ))}
          </ul>
          <h2>Presented claims</h2>
          <p className="muted">Holder: {state.subject}</p>
          <pre>{JSON.stringify(state.claims, null, 2)}</pre>
        </div>
      )}
    </>
  );
}
