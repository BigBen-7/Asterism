#![no_std]
//! Attestation registry: the shared foundation of Asterism.
//!
//! The data model is defined in RFC 0001 and the TTL policy in RFC 0002.
//!
//! # Storage layout
//!
//! | Key                    | Storage    | Value         | Written by                |
//! | ---------------------- | ---------- | ------------- | ------------------------- |
//! | `SchemaCount`          | Instance   | `u64`         | `register_schema`         |
//! | `AttestationCount`     | Instance   | `u64`         | `issue`, `issue_batch`    |
//! | `Schema(SchemaId)`     | Persistent | `Schema`      | `register_schema`         |
//! | `Attestation(AttId)`   | Persistent | `Attestation` | `issue`, `issue_batch`, `revoke` |
//!
//! Ids come from the two counters, start at 1 and are never reused. Schemas
//! are immutable. Attestations change only when `revoked_at` is set. Every
//! write extends the instance TTL and the TTL of the entry it creates or
//! changes to `DEFAULT_TTL_LEDGERS`.
//!
//! Events: `schema_registered`, `issued`, `batch_issued`, `revoked`.

mod merkle;

use asterism_types::{
    Attestation, AttestationId, Commitment, Error, Schema, SchemaId, Status, MAX_NAME_LEN,
    MAX_PROOF_LEN,
};
use soroban_sdk::{
    contract, contractevent, contractimpl, contracttype, Address, Bytes, BytesN, Env, String, Vec,
};

/// Extend an entry when its remaining TTL drops below this (about 30 days).
/// Placeholder per RFC 0002 until issue 0.9 fixes the final values.
const TTL_THRESHOLD: u32 = 518_400;
/// Extend an entry to this many ledgers (about 180 days). Placeholder per RFC 0002.
const DEFAULT_TTL_LEDGERS: u32 = 3_110_400;

#[contracttype]
#[derive(Clone)]
enum DataKey {
    SchemaCount,
    AttestationCount,
    Schema(SchemaId),
    Attestation(AttestationId),
}

#[contractevent]
pub struct SchemaRegistered {
    #[topic]
    pub id: SchemaId,
    pub creator: Address,
}

#[contractevent]
pub struct Issued {
    #[topic]
    pub id: AttestationId,
    pub schema_id: SchemaId,
    pub issuer: Address,
    pub subject: Address,
    pub commitment: BytesN<32>,
}

#[contractevent]
pub struct BatchIssued {
    #[topic]
    pub id: AttestationId,
    pub schema_id: SchemaId,
    pub issuer: Address,
    pub root: BytesN<32>,
}

#[contractevent]
pub struct Revoked {
    #[topic]
    pub id: AttestationId,
    pub issuer: Address,
}

#[contract]
pub struct AttestationRegistry;

#[contractimpl]
impl AttestationRegistry {
    /// Returns the contract interface version.
    pub fn version() -> u32 {
        asterism_types::INTERFACE_VERSION
    }

    /// Registers an immutable schema and returns its id. Anyone may register
    /// a schema; `creator` must authorize the call.
    pub fn register_schema(
        env: Env,
        creator: Address,
        name: String,
        version: u32,
        fields_hash: BytesN<32>,
        revocable: bool,
    ) -> Result<SchemaId, Error> {
        creator.require_auth();
        if name.len() > MAX_NAME_LEN {
            return Err(Error::NameTooLong);
        }

        let id = next_id(&env, DataKey::SchemaCount);
        let schema = Schema {
            name,
            version,
            fields_hash,
            revocable,
            creator: creator.clone(),
        };
        save_persistent(&env, &DataKey::Schema(id), &schema);
        SchemaRegistered { id, creator }.publish(&env);
        Ok(id)
    }

    /// Returns a schema, or `SchemaNotFound`.
    pub fn get_schema(env: Env, id: SchemaId) -> Result<Schema, Error> {
        load_schema(&env, id)
    }

    /// Issues a single attestation about `subject` under `schema_id`.
    /// `issuer` must authorize the call. `expires_at`, if set, must be in the future.
    pub fn issue(
        env: Env,
        issuer: Address,
        schema_id: SchemaId,
        subject: Address,
        data_hash: BytesN<32>,
        expires_at: Option<u64>,
    ) -> Result<AttestationId, Error> {
        issuer.require_auth();
        load_schema(&env, schema_id)?;
        check_expiry(&env, expires_at)?;

        let attestation = Attestation {
            schema_id,
            issuer: issuer.clone(),
            subject: Some(subject.clone()),
            commitment: Commitment::Single(data_hash.clone()),
            issued_at: env.ledger().timestamp(),
            expires_at,
            revoked_at: None,
        };
        let id = store_new_attestation(&env, &attestation);
        Issued {
            id,
            schema_id,
            issuer,
            subject,
            commitment: data_hash,
        }
        .publish(&env);
        Ok(id)
    }

    /// Issues a batch as one record whose commitment is a Merkle root (see
    /// the `merkle` module for the hashing scheme). Batches have no subject.
    pub fn issue_batch(
        env: Env,
        issuer: Address,
        schema_id: SchemaId,
        root: BytesN<32>,
        expires_at: Option<u64>,
    ) -> Result<AttestationId, Error> {
        issuer.require_auth();
        load_schema(&env, schema_id)?;
        check_expiry(&env, expires_at)?;

        let attestation = Attestation {
            schema_id,
            issuer: issuer.clone(),
            subject: None,
            commitment: Commitment::Batch(root.clone()),
            issued_at: env.ledger().timestamp(),
            expires_at,
            revoked_at: None,
        };
        let id = store_new_attestation(&env, &attestation);
        BatchIssued {
            id,
            schema_id,
            issuer,
            root,
        }
        .publish(&env);
        Ok(id)
    }

    /// Returns an attestation, or `AttestationNotFound`.
    pub fn get(env: Env, id: AttestationId) -> Result<Attestation, Error> {
        load_attestation(&env, id)
    }

    /// Returns the derived status (valid, expired or revoked) at the current ledger time.
    pub fn status(env: Env, id: AttestationId) -> Result<Status, Error> {
        Ok(load_attestation(&env, id)?.status(env.ledger().timestamp()))
    }

    /// True if the attestation exists and is currently valid.
    pub fn verify(env: Env, id: AttestationId) -> Result<bool, Error> {
        Ok(Self::status(env, id)? == Status::Valid)
    }

    /// Revokes an attestation. Only the issuer may revoke, only under a
    /// revocable schema, and only once. Revocation sets `revoked_at`; it never deletes.
    pub fn revoke(env: Env, caller: Address, id: AttestationId) -> Result<(), Error> {
        caller.require_auth();
        let mut attestation = load_attestation(&env, id)?;
        if caller != attestation.issuer {
            return Err(Error::NotIssuer);
        }
        if !load_schema(&env, attestation.schema_id)?.revocable {
            return Err(Error::NotRevocable);
        }
        if attestation.revoked_at.is_some() {
            return Err(Error::AlreadyRevoked);
        }

        attestation.revoked_at = Some(env.ledger().timestamp());
        save_persistent(&env, &DataKey::Attestation(id), &attestation);
        Revoked { id, issuer: caller }.publish(&env);
        Ok(())
    }

    /// Checks that `leaf` is a member of batch attestation `id` using `proof`
    /// (sibling hashes from leaf to root).
    ///
    /// Returns `Ok(true)` for a member of a currently valid batch, `Ok(false)`
    /// if the batch is expired or revoked, `Err(InvalidProof)` if the proof does
    /// not lead to the stored root (or the record is not a batch) and
    /// `Err(ProofTooLong)` if the proof exceeds `MAX_PROOF_LEN` levels.
    pub fn verify_inclusion(
        env: Env,
        id: AttestationId,
        leaf: Bytes,
        proof: Vec<BytesN<32>>,
    ) -> Result<bool, Error> {
        if proof.len() > MAX_PROOF_LEN {
            return Err(Error::ProofTooLong);
        }
        let attestation = load_attestation(&env, id)?;
        let Commitment::Batch(root) = &attestation.commitment else {
            return Err(Error::InvalidProof);
        };
        if merkle::compute_root(&env, &leaf, &proof) != *root {
            return Err(Error::InvalidProof);
        }
        Ok(attestation.status(env.ledger().timestamp()) == Status::Valid)
    }
}

fn next_id(env: &Env, key: DataKey) -> u64 {
    let storage = env.storage().instance();
    let id = storage.get::<_, u64>(&key).unwrap_or(0).saturating_add(1);
    storage.set(&key, &id);
    storage.extend_ttl(TTL_THRESHOLD, DEFAULT_TTL_LEDGERS);
    id
}

fn save_persistent<V: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(
    env: &Env,
    key: &DataKey,
    value: &V,
) {
    let storage = env.storage().persistent();
    storage.set(key, value);
    storage.extend_ttl(key, TTL_THRESHOLD, DEFAULT_TTL_LEDGERS);
}

fn load_schema(env: &Env, id: SchemaId) -> Result<Schema, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Schema(id))
        .ok_or(Error::SchemaNotFound)
}

fn load_attestation(env: &Env, id: AttestationId) -> Result<Attestation, Error> {
    env.storage()
        .persistent()
        .get(&DataKey::Attestation(id))
        .ok_or(Error::AttestationNotFound)
}

fn store_new_attestation(env: &Env, attestation: &Attestation) -> AttestationId {
    let id = next_id(env, DataKey::AttestationCount);
    save_persistent(env, &DataKey::Attestation(id), attestation);
    id
}

/// `expires_at`, if set, must be strictly in the future (RFC 0001, decision 4).
fn check_expiry(env: &Env, expires_at: Option<u64>) -> Result<(), Error> {
    match expires_at {
        Some(t) if t <= env.ledger().timestamp() => Err(Error::InvalidExpiry),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod test;
