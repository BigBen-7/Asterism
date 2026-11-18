import { expect, test } from "vitest";
import { StrKey, scValToNative } from "@stellar/stellar-sdk";
import { callArgs, registerSchemaArgs, revokeArgs } from "./calls";
import type { ContractCall } from "@asterism/sdk";

const issuer = StrKey.encodeEd25519PublicKey(new Uint8Array(32).fill(1));
const subject = StrKey.encodeEd25519PublicKey(new Uint8Array(32).fill(2));

test("issue arguments follow the credentials contract signature", () => {
  const call: ContractCall = {
    contract: "C",
    function: "issue",
    args: { issuer, schema_id: 3, subject, data_hash: "ab".repeat(32), expires_at: 5000 },
  };
  const args = callArgs(call).map((a) => scValToNative(a));
  expect(args[0]).toBe(issuer);
  expect(args[1]).toBe(3n);
  expect(args[2]).toBe(subject);
  expect(Array.from(args[3] as Uint8Array)).toEqual(new Array(32).fill(0xab));
  expect(args[4]).toBe(5000n);
});

test("a missing expiry becomes void", () => {
  const call: ContractCall = {
    contract: "C",
    function: "issue",
    args: { issuer, schema_id: 3, subject, data_hash: "00".repeat(32), expires_at: null },
  };
  expect(callArgs(call)[4]?.switch().name).toBe("scvVoid");
});

test("issue_batch arguments", () => {
  const call: ContractCall = {
    contract: "C",
    function: "issue_batch",
    args: { issuer, schema_id: 3, root: "cd".repeat(32), expires_at: null },
  };
  expect(callArgs(call)).toHaveLength(4);
});

test("unknown functions are rejected", () => {
  expect(() => callArgs({ contract: "C", function: "drain", args: {} })).toThrow();
});

test("register_schema and revoke arguments", () => {
  const schema = registerSchemaArgs(issuer, "degree", 1, "11".repeat(32), true).map((a) =>
    scValToNative(a),
  );
  expect(schema[1]).toBe("degree");
  expect(schema[2]).toBe(1);
  expect(schema[4]).toBe(true);
  expect(revokeArgs(issuer, 9).map((a) => scValToNative(a))).toEqual([issuer, 9n]);
});
