import { fromHex, type ContractCall } from "@asterism/sdk";
import { nativeToScVal, xdr } from "@stellar/stellar-sdk";

const address = (v: unknown) => nativeToScVal(String(v), { type: "address" });
const u64 = (v: unknown) => nativeToScVal(BigInt(String(v)), { type: "u64" });
const u32 = (v: number) => nativeToScVal(v, { type: "u32" });
const text = (v: string) => nativeToScVal(v, { type: "string" });
const bytes = (hex: unknown) => nativeToScVal(fromHex(String(hex)));
const optionU64 = (v: unknown) => (v === null || v === undefined ? xdr.ScVal.scvVoid() : u64(v));

/** Contract arguments for a call prepared by the API (`issue` or `issue_batch`). */
export function callArgs(call: ContractCall): xdr.ScVal[] {
  const a = call.args;
  switch (call.function) {
    case "issue":
      return [
        address(a.issuer),
        u64(a.schema_id),
        address(a.subject),
        bytes(a.data_hash),
        optionU64(a.expires_at),
      ];
    case "issue_batch":
      return [address(a.issuer), u64(a.schema_id), bytes(a.root), optionU64(a.expires_at)];
    default:
      throw new Error(`unsupported contract function: ${call.function}`);
  }
}

/** `registry.register_schema(creator, name, version, fields_hash, revocable)` */
export function registerSchemaArgs(
  creator: string,
  name: string,
  version: number,
  fieldsHashHex: string,
  revocable: boolean,
): xdr.ScVal[] {
  return [
    address(creator),
    text(name),
    u32(version),
    bytes(fieldsHashHex),
    nativeToScVal(revocable),
  ];
}

/** `credentials.register_issuer(issuer, name, metadata_hash)` */
export function registerIssuerArgs(
  issuer: string,
  name: string,
  metadataHashHex: string,
): xdr.ScVal[] {
  return [address(issuer), text(name), bytes(metadataHashHex)];
}

/** `credentials.authorize_issuer(creator, schema_id, issuer)` */
export function authorizeIssuerArgs(
  creator: string,
  schemaId: number,
  issuer: string,
): xdr.ScVal[] {
  return [address(creator), u64(schemaId), address(issuer)];
}

/** `credentials.revoke(caller, id)` */
export function revokeArgs(caller: string, attestationId: number): xdr.ScVal[] {
  return [address(caller), u64(attestationId)];
}
