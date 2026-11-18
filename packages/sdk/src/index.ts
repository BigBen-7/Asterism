/** Asterism TypeScript SDK: prepare credentials, build links and verify against the credentials API. */
export const SDK_VERSION = "0.0.0";

export * from "./types";
export { ApiError, createClient, issue, issueBatch, makeVerifyLink, verify } from "./client";
export type { Client, ClientOptions } from "./client";
export { canonicalize, fromHex, schemaFieldsHash, sha256Hex, toHex } from "./canonical";
export { PAYLOAD_VERSION, buildVerifyLink, parseVerifyLink } from "./link";
