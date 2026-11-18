/** Shapes of the credentials API (`docs/api.md`). Field names match the API's JSON. */

export type FieldType = "string" | "integer" | "boolean" | "date" | "datetime" | "address";

export interface SchemaDocument {
  asterism: "credential-schema/1";
  name: string;
  version: number;
  description?: string;
  fields: Record<string, { type: FieldType; required: boolean }>;
}

export interface CredentialRecord {
  schema: { registry: string; id: number };
  subject: string;
  claims: Record<string, unknown>;
  /** 32 random bytes, hex encoded. */
  salt: string;
}

/** A contract call for the issuer to sign and submit. */
export interface ContractCall {
  contract: string;
  function: string;
  args: Record<string, unknown>;
}

export interface PreparedCredential {
  credential: CredentialRecord;
  /** Hex SHA-256 of the canonical credential record. */
  commitment: string;
  call: ContractCall;
}

export interface PreparedBatch {
  /** Hex Merkle root. */
  root: string;
  credentials: { credential: CredentialRecord; proof: string[] }[];
  call: ContractCall;
}

export interface VerifyPayload {
  v: 1;
  attestation_id: number;
  schema_document: SchemaDocument;
  credential: CredentialRecord;
  /** Hex sibling hashes for batch credentials. */
  proof?: string[];
}

export interface VerifyCheck {
  name: string;
  passed: boolean;
  detail?: string;
}

export interface VerifyResult {
  valid: boolean;
  status?: "valid" | "expired" | "revoked";
  checks: VerifyCheck[];
}

export interface IssueRequest {
  /** Stellar address of the issuer who will sign the call. */
  issuer: string;
  schemaId: number;
  schemaDocument: SchemaDocument;
  subject: string;
  claims: Record<string, unknown>;
  /** Unix seconds; omit for a credential that does not expire. */
  expiresAt?: number;
}

export interface IssueBatchRequest {
  issuer: string;
  schemaId: number;
  schemaDocument: SchemaDocument;
  credentials: { subject: string; claims: Record<string, unknown> }[];
  expiresAt?: number;
}

/** What to verify: a payload object, or a link / QR content. */
export type VerifyInput = { payload: VerifyPayload } | { link: string };
