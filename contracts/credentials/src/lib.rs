#![no_std]
//! Credentials module: issuer registration and authorization.
//!
//! The credentials module depends only on the attestation registry. It adds
//! two things the registry deliberately leaves out (RFC 0001, decisions 4 and
//! 2): which addresses are known credential **issuers**, and which issuers a
//! schema's creator has **authorized** to issue under that schema. It then
//! issues, revokes and verifies credentials through the registry, enforcing
//! that authorization on issuance.
//!
//! # Model
//!
//! - An issuer registers itself (`register_issuer`) with a name and the hash
//!   of its off-chain metadata. Anyone may register; registration is identity
//!   only and grants no right to issue.
//! - The creator of a registry schema (as recorded by the registry) decides
//!   who may issue under it: `authorize_issuer` and `deauthorize_issuer`.
//!   The creator is read from the registry, so ownership cannot be claimed
//!   here.
//! - `is_authorized(schema_id, issuer)` is true only for a registered issuer
//!   that the schema's creator has authorized.
//! - `issue` and `issue_batch` require `is_authorized`, then call the registry
//!   with the real issuer as the attestation's issuer. The new attestation id
//!   is recorded here, so `verify` and `verify_inclusion` accept only
//!   credentials issued through this contract (an attestation written to the
//!   registry directly is not a credential).
//! - `revoke` forwards to the registry, which allows only the issuer to revoke
//!   and only under a revocable schema. It does not require the issuer to still
//!   be authorized: deauthorizing an issuer stops new issuance but neither
//!   revokes nor invalidates credentials it already issued. The schema creator
//!   cannot revoke them either; only the issuer can.
//!
//! # Storage layout
//!
//! | Key                         | Storage    | Value    | Written by                              |
//! | --------------------------- | ---------- | -------- | --------------------------------------- |
//! | `Registry`                  | Instance   | `Address`| constructor                             |
//! | `Issuer(Address)`           | Persistent | `Issuer` | `register_issuer`                       |
//! | `Authorized(SchemaId, Address)` | Persistent | `bool` (always `true`) | `authorize_issuer`, `deauthorize_issuer` (removes) |
//! | `Credential(AttestationId)` | Persistent | `bool` (always `true`) | `issue`, `issue_batch` |
//!
//! Records get the registry's default TTL when written (RFC 0002).
//!
//! Events: `issuer_registered`, `issuer_authorized`, `issuer_deauthorized`.

use asterism_types::{Attestation, AttestationId, Schema, SchemaId, Status, MAX_NAME_LEN};
use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, Address,
    Bytes, BytesN, Env, String, Vec,
};

/// Placeholder TTL values from RFC 0002, matching the registry.
const TTL_THRESHOLD: u32 = 518_400;
const DEFAULT_TTL_LEDGERS: u32 = 3_110_400;

/// The part of the attestation registry this contract uses.
#[contractclient(name = "RegistryClient")]
pub trait RegistryInterface {
    fn get_schema(env: Env, id: SchemaId) -> Result<Schema, asterism_types::Error>;
    fn issue(
        env: Env,
        issuer: Address,
        schema_id: SchemaId,
        subject: Address,
        data_hash: BytesN<32>,
        expires_at: Option<u64>,
    ) -> Result<AttestationId, asterism_types::Error>;
    fn issue_batch(
        env: Env,
        issuer: Address,
        schema_id: SchemaId,
        root: BytesN<32>,
        expires_at: Option<u64>,
    ) -> Result<AttestationId, asterism_types::Error>;
    fn get(env: Env, id: AttestationId) -> Result<Attestation, asterism_types::Error>;
    fn status(env: Env, id: AttestationId) -> Result<Status, asterism_types::Error>;
    fn revoke(env: Env, caller: Address, id: AttestationId) -> Result<(), asterism_types::Error>;
    fn verify_inclusion(
        env: Env,
        id: AttestationId,
        leaf: Bytes,
        proof: Vec<BytesN<32>>,
    ) -> Result<bool, asterism_types::Error>;
}

/// A registered credential issuer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Issuer {
    pub name: String,
    /// SHA-256 of the issuer's off-chain metadata document.
    pub metadata_hash: BytesN<32>,
    /// Ledger timestamp at registration.
    pub registered_at: u64,
}

/// Credentials errors. Codes are stable and append-only.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    AlreadyRegistered = 1,
    NotRegistered = 2,
    NameTooLong = 3,
    SchemaNotFound = 4,
    NotSchemaCreator = 5,
    AlreadyAuthorized = 6,
    NotAuthorized = 7,
    /// Issuer is not registered and authorized for the schema.
    IssuerNotAuthorized = 8,
    /// The attestation was not issued through this contract (or does not exist).
    NotACredential = 9,
    // Registry errors, surfaced with the same meaning.
    InvalidExpiry = 10,
    NotRevocable = 11,
    AlreadyRevoked = 12,
    NotIssuer = 13,
    InvalidProof = 14,
    ProofTooLong = 15,
    /// Any other registry failure.
    RegistryFailed = 16,
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Registry,
    Issuer(Address),
    Authorized(SchemaId, Address),
    Credential(AttestationId),
}

#[contractevent]
pub struct IssuerRegistered {
    #[topic]
    pub issuer: Address,
}

#[contractevent]
pub struct IssuerAuthorized {
    #[topic]
    pub schema_id: SchemaId,
    #[topic]
    pub issuer: Address,
}

#[contractevent]
pub struct IssuerDeauthorized {
    #[topic]
    pub schema_id: SchemaId,
    #[topic]
    pub issuer: Address,
}

#[contract]
pub struct Credentials;

#[contractimpl]
impl Credentials {
    /// Binds this contract to one attestation registry for its lifetime.
    pub fn __constructor(env: Env, registry: Address) {
        env.storage().instance().set(&DataKey::Registry, &registry);
    }

    /// The attestation registry this contract reads schemas from.
    pub fn registry(env: Env) -> Result<Address, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Registry)
            .ok_or(Error::SchemaNotFound)
    }

    /// Registers `issuer` with a name and metadata hash. `issuer` must authorize the call.
    /// Registration is identity only; it does not allow issuing under any schema.
    pub fn register_issuer(
        env: Env,
        issuer: Address,
        name: String,
        metadata_hash: BytesN<32>,
    ) -> Result<(), Error> {
        issuer.require_auth();
        if name.len() > MAX_NAME_LEN {
            return Err(Error::NameTooLong);
        }
        let key = DataKey::Issuer(issuer.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::AlreadyRegistered);
        }
        let record = Issuer {
            name,
            metadata_hash,
            registered_at: env.ledger().timestamp(),
        };
        save(&env, &key, &record);
        IssuerRegistered { issuer }.publish(&env);
        Ok(())
    }

    /// Returns a registered issuer, or `NotRegistered`.
    pub fn get_issuer(env: Env, issuer: Address) -> Result<Issuer, Error> {
        env.storage()
            .persistent()
            .get(&DataKey::Issuer(issuer))
            .ok_or(Error::NotRegistered)
    }

    /// Lets `issuer` issue under `schema_id`. `creator` must be the schema's
    /// creator as recorded in the registry and must authorize the call. The
    /// issuer must already be registered.
    pub fn authorize_issuer(
        env: Env,
        creator: Address,
        schema_id: SchemaId,
        issuer: Address,
    ) -> Result<(), Error> {
        creator.require_auth();
        check_schema_creator(&env, schema_id, &creator)?;
        if !env
            .storage()
            .persistent()
            .has(&DataKey::Issuer(issuer.clone()))
        {
            return Err(Error::NotRegistered);
        }
        let key = DataKey::Authorized(schema_id, issuer.clone());
        if env.storage().persistent().has(&key) {
            return Err(Error::AlreadyAuthorized);
        }
        save(&env, &key, &true);
        IssuerAuthorized { schema_id, issuer }.publish(&env);
        Ok(())
    }

    /// Removes `issuer`'s right to issue under `schema_id`. Same authorization
    /// as `authorize_issuer`. Attestations already issued are not affected.
    pub fn deauthorize_issuer(
        env: Env,
        creator: Address,
        schema_id: SchemaId,
        issuer: Address,
    ) -> Result<(), Error> {
        creator.require_auth();
        check_schema_creator(&env, schema_id, &creator)?;
        let key = DataKey::Authorized(schema_id, issuer.clone());
        if !env.storage().persistent().has(&key) {
            return Err(Error::NotAuthorized);
        }
        env.storage().persistent().remove(&key);
        IssuerDeauthorized { schema_id, issuer }.publish(&env);
        Ok(())
    }

    /// Issues a credential about `subject` under `schema_id`. `issuer` must
    /// authorize the call, be registered, and be authorized by the schema's
    /// creator. `data_hash` and `expires_at` are as in the registry's `issue`.
    pub fn issue(
        env: Env,
        issuer: Address,
        schema_id: SchemaId,
        subject: Address,
        data_hash: BytesN<32>,
        expires_at: Option<u64>,
    ) -> Result<AttestationId, Error> {
        issuer.require_auth();
        if !Self::is_authorized(env.clone(), schema_id, issuer.clone()) {
            return Err(Error::IssuerNotAuthorized);
        }
        let id =
            match registry(&env)?.try_issue(&issuer, &schema_id, &subject, &data_hash, &expires_at)
            {
                Ok(Ok(id)) => id,
                Err(Ok(e)) => return Err(map_registry_error(e)),
                _ => return Err(Error::RegistryFailed),
            };
        save(&env, &DataKey::Credential(id), &true);
        Ok(id)
    }

    /// Issues a batch of credentials as one Merkle root. Same authorization
    /// as `issue`. See the registry for the hashing scheme.
    pub fn issue_batch(
        env: Env,
        issuer: Address,
        schema_id: SchemaId,
        root: BytesN<32>,
        expires_at: Option<u64>,
    ) -> Result<AttestationId, Error> {
        issuer.require_auth();
        if !Self::is_authorized(env.clone(), schema_id, issuer.clone()) {
            return Err(Error::IssuerNotAuthorized);
        }
        let id = match registry(&env)?.try_issue_batch(&issuer, &schema_id, &root, &expires_at) {
            Ok(Ok(id)) => id,
            Err(Ok(e)) => return Err(map_registry_error(e)),
            _ => return Err(Error::RegistryFailed),
        };
        save(&env, &DataKey::Credential(id), &true);
        Ok(id)
    }

    /// Revokes a credential. `caller` must be its issuer; the registry also
    /// requires a revocable schema and rejects a second revocation.
    pub fn revoke(env: Env, caller: Address, id: AttestationId) -> Result<(), Error> {
        caller.require_auth();
        ensure_credential(&env, id)?;
        match registry(&env)?.try_revoke(&caller, &id) {
            Ok(Ok(())) => Ok(()),
            Err(Ok(e)) => Err(map_registry_error(e)),
            _ => Err(Error::RegistryFailed),
        }
    }

    /// The credential's status (valid, expired or revoked) at the current ledger time.
    pub fn status(env: Env, id: AttestationId) -> Result<Status, Error> {
        ensure_credential(&env, id)?;
        match registry(&env)?.try_status(&id) {
            Ok(Ok(status)) => Ok(status),
            Err(Ok(e)) => Err(map_registry_error(e)),
            _ => Err(Error::RegistryFailed),
        }
    }

    /// True if `id` is a credential issued through this contract and is currently valid.
    /// Credentials issued by a since-deauthorized issuer stay valid.
    pub fn verify(env: Env, id: AttestationId) -> Result<bool, Error> {
        Ok(Self::status(env, id)? == Status::Valid)
    }

    /// Checks that `leaf` belongs to the batch credential `id` using `proof`.
    /// Results are as in the registry's `verify_inclusion`: `true` for a member
    /// of a currently valid batch, `false` if the batch is expired or revoked,
    /// and `InvalidProof` / `ProofTooLong` for a bad proof.
    pub fn verify_inclusion(
        env: Env,
        id: AttestationId,
        leaf: Bytes,
        proof: Vec<BytesN<32>>,
    ) -> Result<bool, Error> {
        ensure_credential(&env, id)?;
        match registry(&env)?.try_verify_inclusion(&id, &leaf, &proof) {
            Ok(Ok(valid)) => Ok(valid),
            Err(Ok(e)) => Err(map_registry_error(e)),
            _ => Err(Error::RegistryFailed),
        }
    }

    /// True if `issuer` is registered and authorized under `schema_id`.
    pub fn is_authorized(env: Env, schema_id: SchemaId, issuer: Address) -> bool {
        let storage = env.storage().persistent();
        storage.has(&DataKey::Issuer(issuer.clone()))
            && storage.has(&DataKey::Authorized(schema_id, issuer))
    }
}

fn save<V: soroban_sdk::IntoVal<Env, soroban_sdk::Val>>(env: &Env, key: &DataKey, value: &V) {
    let storage = env.storage().persistent();
    storage.set(key, value);
    storage.extend_ttl(key, TTL_THRESHOLD, DEFAULT_TTL_LEDGERS);
}

fn registry(env: &Env) -> Result<RegistryClient<'_>, Error> {
    let address: Address = env
        .storage()
        .instance()
        .get(&DataKey::Registry)
        .ok_or(Error::RegistryFailed)?;
    Ok(RegistryClient::new(env, &address))
}

fn ensure_credential(env: &Env, id: AttestationId) -> Result<(), Error> {
    if env.storage().persistent().has(&DataKey::Credential(id)) {
        Ok(())
    } else {
        Err(Error::NotACredential)
    }
}

fn map_registry_error(e: asterism_types::Error) -> Error {
    use asterism_types::Error as R;
    match e {
        R::SchemaNotFound => Error::SchemaNotFound,
        R::AttestationNotFound => Error::NotACredential,
        R::InvalidExpiry => Error::InvalidExpiry,
        R::NotRevocable => Error::NotRevocable,
        R::AlreadyRevoked => Error::AlreadyRevoked,
        R::NotIssuer => Error::NotIssuer,
        R::InvalidProof => Error::InvalidProof,
        R::ProofTooLong => Error::ProofTooLong,
        _ => Error::RegistryFailed,
    }
}

/// Reads the schema from the registry and checks that `creator` created it.
fn check_schema_creator(env: &Env, schema_id: SchemaId, creator: &Address) -> Result<(), Error> {
    let registry: Address = env
        .storage()
        .instance()
        .get(&DataKey::Registry)
        .ok_or(Error::SchemaNotFound)?;
    let schema = match RegistryClient::new(env, &registry).try_get_schema(&schema_id) {
        Ok(Ok(schema)) => schema,
        _ => return Err(Error::SchemaNotFound),
    };
    if schema.creator != *creator {
        return Err(Error::NotSchemaCreator);
    }
    Ok(())
}

#[cfg(test)]
mod test;
