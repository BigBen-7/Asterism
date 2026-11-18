/**
 * JSON canonicalization (RFC 8785, JCS) for the subset Asterism hashes:
 * objects, arrays, strings, booleans, null and integers. Floats are rejected.
 * Matches `services/api/src/canonical.rs`.
 */
export function canonicalize(value: unknown): string {
  if (value === null) return "null";
  switch (typeof value) {
    case "boolean":
      return value ? "true" : "false";
    case "string":
      return JSON.stringify(value);
    case "bigint":
      return value.toString();
    case "number":
      if (!Number.isSafeInteger(value)) {
        throw new Error("only safe integers can be canonicalized");
      }
      return String(value);
    case "object": {
      if (Array.isArray(value)) return `[${value.map(canonicalize).join(",")}]`;
      const entries = Object.entries(value as Record<string, unknown>)
        .filter(([, v]) => v !== undefined)
        // Default string order is by UTF-16 code unit, as JCS requires.
        .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
        .map(([k, v]) => `${JSON.stringify(k)}:${canonicalize(v)}`);
      return `{${entries.join(",")}}`;
    }
    default:
      throw new Error(`cannot canonicalize a value of type ${typeof value}`);
  }
}

export function toHex(bytes: Uint8Array): string {
  return Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join("");
}

export function fromHex(hex: string): Uint8Array {
  if (hex.length % 2 !== 0 || !/^[0-9a-fA-F]*$/.test(hex)) {
    throw new Error("invalid hex string");
  }
  const out = new Uint8Array(hex.length / 2);
  for (let i = 0; i < out.length; i++) {
    out[i] = parseInt(hex.slice(i * 2, i * 2 + 2), 16);
  }
  return out;
}

/** Hex SHA-256 of `data` (uses the Web Crypto API, available in browsers and Node 22). */
export async function sha256Hex(data: Uint8Array | string): Promise<string> {
  const bytes = typeof data === "string" ? new TextEncoder().encode(data) : data;
  const digest = await crypto.subtle.digest("SHA-256", bytes as BufferSource);
  return toHex(new Uint8Array(digest));
}

/**
 * The value to register as the registry's `fields_hash` for a schema
 * document: `SHA-256(JCS(document))`, hex encoded.
 */
export function schemaFieldsHash(document: unknown): Promise<string> {
  return sha256Hex(canonicalize(document));
}
