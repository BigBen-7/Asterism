import { canonicalize } from "./canonical";
import type { VerifyPayload } from "./types";

/** Payload format version understood by this SDK. */
export const PAYLOAD_VERSION = 1;

function base64UrlEncode(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let binary = "";
  for (const b of bytes) binary += String.fromCharCode(b);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}

function base64UrlDecode(encoded: string): string {
  if (!/^[A-Za-z0-9_-]*$/.test(encoded)) throw new Error("payload is not valid base64url");
  const padded = encoded.replace(/-/g, "+").replace(/_/g, "/");
  const binary = atob(padded + "=".repeat((4 - (padded.length % 4)) % 4));
  return new TextDecoder().decode(Uint8Array.from(binary, (c) => c.charCodeAt(0)));
}

/**
 * Builds the verification link (and QR content) for a payload:
 * `<baseUrl>/verify#<base64url(canonical JSON)>`. The payload sits in the URL
 * fragment, so browsers never send it to a server.
 */
export function buildVerifyLink(baseUrl: string, payload: VerifyPayload): string {
  return `${baseUrl.replace(/\/+$/, "")}/verify#${base64UrlEncode(canonicalize(payload))}`;
}

/** Parses a link built by {@link buildVerifyLink} (only the fragment is read). */
export function parseVerifyLink(link: string): VerifyPayload {
  const hash = link.indexOf("#");
  if (hash < 0) throw new Error("link has no payload fragment");
  let parsed: unknown;
  try {
    parsed = JSON.parse(base64UrlDecode(link.slice(hash + 1)));
  } catch {
    throw new Error("link payload is not valid");
  }
  const payload = parsed as Partial<VerifyPayload> | null;
  if (!payload || typeof payload !== "object" || payload.v !== PAYLOAD_VERSION) {
    throw new Error("unsupported payload version");
  }
  return payload as VerifyPayload;
}
