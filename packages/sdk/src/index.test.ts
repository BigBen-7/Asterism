import { describe, expect, test } from "vitest";
import {
  ApiError,
  SDK_VERSION,
  buildVerifyLink,
  canonicalize,
  createClient,
  fromHex,
  issue,
  issueBatch,
  makeVerifyLink,
  parseVerifyLink,
  schemaFieldsHash,
  sha256Hex,
  toHex,
  verify,
  type SchemaDocument,
  type VerifyPayload,
} from "./index";

const schema: SchemaDocument = {
  asterism: "credential-schema/1",
  name: "University degree",
  version: 1,
  fields: {
    degree: { type: "string", required: true },
    graduated: { type: "date", required: true },
  },
};

const payload: VerifyPayload = {
  v: 1,
  attestation_id: 12,
  schema_document: schema,
  credential: {
    schema: { registry: "CREGISTRY", id: 3 },
    subject: "GHOLDER",
    claims: { degree: "BSc", graduated: "2026-07-15" },
    salt: "ab".repeat(32),
  },
};

interface Recorded {
  url: string;
  init: RequestInit;
}

/** A fetch that records requests and answers with `status` and `body`. */
function fakeFetch(status: number, body: unknown, calls: Recorded[] = []): typeof fetch {
  return async (input, init) => {
    calls.push({ url: String(input), init: init ?? {} });
    return new Response(JSON.stringify(body), {
      status,
      headers: { "content-type": "application/json" },
    });
  };
}

test("exports a version", () => {
  expect(SDK_VERSION).toBe("0.0.0");
});

describe("canonicalize", () => {
  test("sorts keys and drops whitespace like the Rust implementation", () => {
    expect(canonicalize({ b: 1, a: { d: [true, null], c: "x" } })).toBe(
      '{"a":{"c":"x","d":[true,null]},"b":1}',
    );
  });

  test("is independent of key insertion order", () => {
    expect(canonicalize({ x: 1, y: 2 })).toBe(canonicalize({ y: 2, x: 1 }));
  });

  test("escapes control characters", () => {
    expect(canonicalize({ s: 'a"b\\c\n\u0001é' })).toBe('{"s":"a\\"b\\\\c\\n\\u0001é"}');
  });

  test("rejects floats and omits undefined properties", () => {
    expect(() => canonicalize({ n: 1.5 })).toThrow();
    expect(canonicalize({ a: 1, b: undefined })).toBe('{"a":1}');
  });
});

describe("hashing", () => {
  test("sha256 matches the known vector for 'abc'", async () => {
    expect(await sha256Hex("abc")).toBe(
      "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    );
  });

  test("hex round trip", () => {
    const bytes = new Uint8Array([0, 15, 255]);
    expect(toHex(bytes)).toBe("000fff");
    expect(Array.from(fromHex("000fff"))).toEqual([0, 15, 255]);
    expect(() => fromHex("zz")).toThrow();
  });

  test("schemaFieldsHash does not depend on key order", async () => {
    const reordered = {
      version: 1,
      fields: {
        graduated: { required: true, type: "date" },
        degree: { required: true, type: "string" },
      },
      name: "University degree",
      asterism: "credential-schema/1",
    };
    expect(await schemaFieldsHash(schema)).toBe(await schemaFieldsHash(reordered));
    expect(await schemaFieldsHash({ ...schema, version: 2 })).not.toBe(
      await schemaFieldsHash(schema),
    );
  });
});

describe("links", () => {
  test("round trip", () => {
    const link = buildVerifyLink("https://verify.example/", payload);
    expect(link.startsWith("https://verify.example/verify#")).toBe(true);
    expect(parseVerifyLink(link)).toEqual(payload);
  });

  test("the same payload gives the same link", () => {
    expect(buildVerifyLink("https://v.example", payload)).toBe(
      buildVerifyLink("https://v.example", { ...payload }),
    );
  });

  test("non-ASCII claims survive", () => {
    const p = {
      ...payload,
      credential: { ...payload.credential, claims: { degree: "Ingénieur" } },
    };
    expect(parseVerifyLink(buildVerifyLink("https://v.example", p)).credential.claims).toEqual({
      degree: "Ingénieur",
    });
  });

  test("bad links are rejected", () => {
    expect(() => parseVerifyLink("https://v.example/verify")).toThrow();
    expect(() => parseVerifyLink("https://v.example/verify#%%%")).toThrow();
    expect(() => parseVerifyLink("x#aGVsbG8")).toThrow();
    const wrongVersion = buildVerifyLink("https://v.example", { ...payload, v: 2 as unknown as 1 });
    expect(() => parseVerifyLink(wrongVersion)).toThrow(/version/);
  });
});

describe("client", () => {
  test("issue posts snake_case fields and returns the prepared credential", async () => {
    const calls: Recorded[] = [];
    const prepared = {
      credential: payload.credential,
      commitment: "11".repeat(32),
      call: { contract: "CCRED", function: "issue", args: {} },
    };
    const client = createClient({
      apiUrl: "http://api.test/",
      fetch: fakeFetch(200, prepared, calls),
    });

    const result = await issue(client, {
      issuer: "GISSUER",
      schemaId: 3,
      schemaDocument: schema,
      subject: "GHOLDER",
      claims: { degree: "BSc", graduated: "2026-07-15" },
      expiresAt: 5000,
    });

    expect(result).toEqual(prepared);
    expect(calls[0]?.url).toBe("http://api.test/v1/credentials");
    expect(calls[0]?.init.method).toBe("POST");
    expect(JSON.parse(String(calls[0]?.init.body))).toEqual({
      issuer: "GISSUER",
      schema_id: 3,
      schema_document: schema,
      subject: "GHOLDER",
      claims: { degree: "BSc", graduated: "2026-07-15" },
      expires_at: 5000,
    });
  });

  test("issue sends a null expiry when none is given", async () => {
    const calls: Recorded[] = [];
    const client = createClient({ apiUrl: "http://api.test", fetch: fakeFetch(200, {}, calls) });
    await issue(client, {
      issuer: "G",
      schemaId: 1,
      schemaDocument: schema,
      subject: "G",
      claims: {},
    });
    expect(JSON.parse(String(calls[0]?.init.body)).expires_at).toBeNull();
  });

  test("issueBatch uses the batch endpoint", async () => {
    const calls: Recorded[] = [];
    const client = createClient({ apiUrl: "http://api.test", fetch: fakeFetch(200, {}, calls) });
    await issueBatch(client, {
      issuer: "G",
      schemaId: 1,
      schemaDocument: schema,
      credentials: [{ subject: "G1", claims: { degree: "BSc" } }],
    });
    expect(calls[0]?.url).toBe("http://api.test/v1/credentials/batch");
    expect(JSON.parse(String(calls[0]?.init.body)).credentials).toHaveLength(1);
  });

  test("verify sends a payload or a link", async () => {
    const calls: Recorded[] = [];
    const result = { valid: true, status: "valid", checks: [] };
    const client = createClient({
      apiUrl: "http://api.test",
      fetch: fakeFetch(200, result, calls),
    });

    expect(await verify(client, { payload })).toEqual(result);
    expect(await verify(client, { link: "https://v.example/verify#abc" })).toEqual(result);
    expect(calls.map((c) => c.url)).toEqual([
      "http://api.test/v1/verify",
      "http://api.test/v1/verify",
    ]);
    expect(JSON.parse(String(calls[1]?.init.body))).toEqual({
      link: "https://v.example/verify#abc",
    });
  });

  test("a credential that fails verification resolves, it does not throw", async () => {
    const client = createClient({
      apiUrl: "http://api.test",
      fetch: fakeFetch(200, { valid: false, checks: [{ name: "commitment", passed: false }] }),
    });
    const result = await verify(client, { payload });
    expect(result.valid).toBe(false);
  });

  test("API errors become ApiError with the server's message", async () => {
    const client = createClient({
      apiUrl: "http://api.test",
      fetch: fakeFetch(400, { error: "invalid credential: missing required field `degree`" }),
    });
    const error = await verify(client, { payload }).catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).status).toBe(400);
    expect((error as ApiError).message).toContain("missing required field");
  });

  test("non-JSON error bodies still produce an ApiError", async () => {
    const client = createClient({
      apiUrl: "http://api.test",
      fetch: async () => new Response("bad gateway", { status: 502, statusText: "Bad Gateway" }),
    });
    const error = await verify(client, { payload }).catch((e: unknown) => e);
    expect(error).toBeInstanceOf(ApiError);
    expect((error as ApiError).message).toBe("Bad Gateway");
  });

  test("makeVerifyLink returns the link from the API", async () => {
    const calls: Recorded[] = [];
    const client = createClient({
      apiUrl: "http://api.test",
      fetch: fakeFetch(200, { link: "https://v.example/verify#xyz" }, calls),
    });
    const rest = {
      attestation_id: payload.attestation_id,
      schema_document: payload.schema_document,
      credential: payload.credential,
    };
    expect(await makeVerifyLink(client, rest)).toBe("https://v.example/verify#xyz");
    expect(calls[0]?.url).toBe("http://api.test/v1/verify/link");
  });
});
