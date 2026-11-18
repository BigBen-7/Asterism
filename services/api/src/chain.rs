//! Reading registry state from the indexer's Postgres tables.
//!
//! The registry emits `schema_registered`, `issued`, `batch_issued` and
//! `revoked` events (see `contracts/attestation-registry/src/lib.rs`). The
//! indexer stores them decoded with `xdrFormat: "json"`, so a map value looks
//! like `{"map":[{"key":{"symbol":"commitment"},"val":{"bytes":"<hex>"}}]}`,
//! a `u64` is `{"u64":"<decimal string>"}`, an address is `{"address":"G..."}`
//! and an absent `Option` is `"void"`. These shapes were checked against live
//! testnet events; the event field names come from the registry source.

use std::future::Future;

use serde_json::Value;
use sqlx::{types::Json, PgPool};

use crate::{merkle::Hash, Error};

/// What an attestation commits to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Commitment {
    Single(Hash),
    Batch(Hash),
}

/// An attestation as reconstructed from events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnChainAttestation {
    pub id: u64,
    pub schema_id: u64,
    pub issuer: String,
    /// `Some` for single attestations.
    pub subject: Option<String>,
    pub commitment: Commitment,
    pub expires_at: Option<u64>,
    pub revoked: bool,
}

/// Source of registry state. Implemented over Postgres; tests use an in-memory one.
pub trait ChainReader: Send + Sync + 'static {
    fn attestation(
        &self,
        id: u64,
    ) -> impl Future<Output = Result<Option<OnChainAttestation>, Error>> + Send;

    fn schema_fields_hash(
        &self,
        schema_id: u64,
    ) -> impl Future<Output = Result<Option<Hash>, Error>> + Send;
}

/// Reads the indexer's `events` table for one registry contract.
pub struct PgChainReader {
    pool: PgPool,
    registry_id: String,
}

impl PgChainReader {
    pub fn new(pool: PgPool, registry_id: String) -> Self {
        Self { pool, registry_id }
    }

    /// The first event with the given name whose first topic after the name is `id`.
    async fn find_event(&self, name: &str, id: u64) -> Result<Option<Value>, Error> {
        let row = sqlx::query_as::<_, (Json<Value>,)>(
            "SELECT data FROM events
             WHERE contract_id = $1 AND topics->0->>'symbol' = $2 AND topics->1->>'u64' = $3
             ORDER BY id LIMIT 1",
        )
        .bind(&self.registry_id)
        .bind(name)
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(Json(data),)| data))
    }
}

impl ChainReader for PgChainReader {
    async fn attestation(&self, id: u64) -> Result<Option<OnChainAttestation>, Error> {
        let single = self.find_event("issued", id).await?;
        let found = match single {
            Some(data) => decode_issued(id, &data),
            None => match self.find_event("batch_issued", id).await? {
                Some(data) => decode_batch_issued(id, &data),
                None => return Ok(None),
            },
        };
        let mut attestation = found
            .ok_or_else(|| Error::Chain(format!("could not decode the issuance event for {id}")))?;
        attestation.revoked = self.find_event("revoked", id).await?.is_some();
        Ok(Some(attestation))
    }

    async fn schema_fields_hash(&self, schema_id: u64) -> Result<Option<Hash>, Error> {
        match self.find_event("schema_registered", schema_id).await? {
            Some(data) => map_field(&data, "fields_hash")
                .and_then(bytes32)
                .map(Some)
                .ok_or_else(|| Error::Chain("could not decode schema_registered".to_string())),
            None => Ok(None),
        }
    }
}

fn symbol(v: &Value) -> Option<&str> {
    v.get("symbol")?.as_str()
}

/// Looks up `name` in a decoded `{"map":[{"key":..,"val":..}]}` value.
fn map_field<'a>(data: &'a Value, name: &str) -> Option<&'a Value> {
    data.get("map")?
        .as_array()?
        .iter()
        .find(|entry| entry.get("key").and_then(symbol) == Some(name))
        .and_then(|entry| entry.get("val"))
}

fn u64_of(v: &Value) -> Option<u64> {
    v.get("u64")?.as_str()?.parse().ok()
}

fn address_of(v: &Value) -> Option<String> {
    v.get("address")?.as_str().map(str::to_string)
}

fn bytes32(v: &Value) -> Option<Hash> {
    hex::decode(v.get("bytes")?.as_str()?).ok()?.try_into().ok()
}

/// `None` for `"void"`, `Some(n)` for `{"u64": ..}`; `Err(())` for anything else.
fn option_u64(v: &Value) -> Result<Option<u64>, ()> {
    if v.as_str() == Some("void") {
        Ok(None)
    } else {
        u64_of(v).map(Some).ok_or(())
    }
}

pub(crate) fn decode_issued(id: u64, data: &Value) -> Option<OnChainAttestation> {
    Some(OnChainAttestation {
        id,
        schema_id: u64_of(map_field(data, "schema_id")?)?,
        issuer: address_of(map_field(data, "issuer")?)?,
        subject: Some(address_of(map_field(data, "subject")?)?),
        commitment: Commitment::Single(bytes32(map_field(data, "commitment")?)?),
        expires_at: option_u64(map_field(data, "expires_at")?).ok()?,
        revoked: false,
    })
}

pub(crate) fn decode_batch_issued(id: u64, data: &Value) -> Option<OnChainAttestation> {
    Some(OnChainAttestation {
        id,
        schema_id: u64_of(map_field(data, "schema_id")?)?,
        issuer: address_of(map_field(data, "issuer")?)?,
        subject: None,
        commitment: Commitment::Batch(bytes32(map_field(data, "root")?)?),
        expires_at: option_u64(map_field(data, "expires_at")?).ok()?,
        revoked: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry(key: &str, val: Value) -> Value {
        json!({ "key": { "symbol": key }, "val": val })
    }

    #[test]
    fn decodes_a_single_issuance_event() {
        let data = json!({ "map": [
            entry("commitment", json!({ "bytes": "11".repeat(32) })),
            entry("expires_at", json!({ "u64": "500" })),
            entry("issuer", json!({ "address": "GISSUER" })),
            entry("schema_id", json!({ "u64": "3" })),
            entry("subject", json!({ "address": "GSUBJECT" })),
        ]});
        let a = decode_issued(8, &data).unwrap();
        assert_eq!(a.id, 8);
        assert_eq!(a.schema_id, 3);
        assert_eq!(a.issuer, "GISSUER");
        assert_eq!(a.subject.as_deref(), Some("GSUBJECT"));
        assert_eq!(a.commitment, Commitment::Single([0x11; 32]));
        assert_eq!(a.expires_at, Some(500));
        assert!(!a.revoked);
    }

    #[test]
    fn decodes_a_batch_event_without_expiry() {
        let data = json!({ "map": [
            entry("expires_at", json!("void")),
            entry("issuer", json!({ "address": "GISSUER" })),
            entry("root", json!({ "bytes": "22".repeat(32) })),
            entry("schema_id", json!({ "u64": "3" })),
        ]});
        let a = decode_batch_issued(9, &data).unwrap();
        assert_eq!(a.subject, None);
        assert_eq!(a.commitment, Commitment::Batch([0x22; 32]));
        assert_eq!(a.expires_at, None);
    }

    #[test]
    fn malformed_events_do_not_decode() {
        assert!(decode_issued(1, &json!({ "map": [] })).is_none());
        assert!(decode_issued(1, &json!("void")).is_none());
        let short_hash = json!({ "map": [
            entry("commitment", json!({ "bytes": "11" })),
            entry("expires_at", json!("void")),
            entry("issuer", json!({ "address": "G" })),
            entry("schema_id", json!({ "u64": "3" })),
            entry("subject", json!({ "address": "G" })),
        ]});
        assert!(decode_issued(1, &short_hash).is_none());
    }
}
