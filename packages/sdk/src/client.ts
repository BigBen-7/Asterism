import type {
  IssueBatchRequest,
  IssueRequest,
  PreparedBatch,
  PreparedCredential,
  VerifyInput,
  VerifyPayload,
  VerifyResult,
} from "./types";

/** An error response from the API. */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export interface ClientOptions {
  /** Base URL of the credentials API, for example `http://localhost:8080`. */
  apiUrl: string;
  /** Custom `fetch`, for tests or non-browser runtimes. Defaults to the global one. */
  fetch?: typeof fetch;
}

export interface Client {
  readonly apiUrl: string;
  post<T>(path: string, body: unknown): Promise<T>;
}

export function createClient(options: ClientOptions): Client {
  const apiUrl = options.apiUrl.replace(/\/+$/, "");
  const doFetch = options.fetch ?? ((...args) => fetch(...args));
  return {
    apiUrl,
    async post<T>(path: string, body: unknown): Promise<T> {
      const response = await doFetch(`${apiUrl}${path}`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
      const text = await response.text();
      let data: unknown;
      try {
        data = text ? JSON.parse(text) : undefined;
      } catch {
        data = undefined;
      }
      if (!response.ok) {
        const message =
          data && typeof data === "object" && "error" in data
            ? String((data as { error: unknown }).error)
            : response.statusText || "request failed";
        throw new ApiError(response.status, message);
      }
      return data as T;
    },
  };
}

/**
 * Prepares a single credential: validates the claims against the schema
 * document, and returns the credential record, its commitment and the
 * `credentials.issue` call for the issuer to sign and submit. The API holds no
 * keys and does not submit anything.
 */
export function issue(client: Client, request: IssueRequest): Promise<PreparedCredential> {
  return client.post("/v1/credentials", {
    issuer: request.issuer,
    schema_id: request.schemaId,
    schema_document: request.schemaDocument,
    subject: request.subject,
    claims: request.claims,
    expires_at: request.expiresAt ?? null,
  });
}

/** Prepares a batch (1 to 1000 credentials) as one Merkle root and a proof per credential. */
export function issueBatch(client: Client, request: IssueBatchRequest): Promise<PreparedBatch> {
  return client.post("/v1/credentials/batch", {
    issuer: request.issuer,
    schema_id: request.schemaId,
    schema_document: request.schemaDocument,
    credentials: request.credentials,
    expires_at: request.expiresAt ?? null,
  });
}

/**
 * Verifies a credential against the chain. Resolves with the result even when
 * the credential is not valid (`valid: false` and the failed checks); rejects
 * with {@link ApiError} for a malformed request.
 */
export function verify(client: Client, input: VerifyInput): Promise<VerifyResult> {
  return client.post("/v1/verify", input);
}

/** Asks the API to build the verification link for a payload. */
export async function makeVerifyLink(
  client: Client,
  payload: Omit<VerifyPayload, "v">,
): Promise<string> {
  const { link } = await client.post<{ link: string }>("/v1/verify/link", payload);
  return link;
}
