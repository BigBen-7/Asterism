#![no_std]
//! Credentials module: issuer registration and authorization.
//!
//! The credentials module depends only on the attestation registry. It adds
//! two things the registry deliberately leaves out (RFC 0001, decisions 4 and
//! 2): which addresses are known credential **issuers**, and which issuers a
//! schema's creator has **authorized** to issue under that schema. Issuing,
//! revoking and verifying credentials come in later issues and will call
//! [`Credentials::is_authorized`].
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
//!
//! # Storage layout
//!
//! | Key                         | Storage    | Value    | Written by                              |
//! | --------------------------- | ---------- | -------- | --------------------------------------- |
//! | `Registry`                  | Instance   | `Address`| constructor                             |
//! | `Issuer(Address)`           | Persistent | `Issuer` | `register_issuer`                       |
//! | `Authorized(SchemaId, Address)` | Persistent | `bool` (always `true`) | `authorize_issuer`, `deauthorize_issuer` (removes) |
//!
//! Records get the registry's default TTL when written (RFC 0002).
//!
//! Events: `issuer_registered`, `issuer_authorized`, `issuer_deauthorized`.

use asterism_types::{Schema, SchemaId, MAX_NAME_LEN};
use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype, Address,
    BytesN, Env, String,
};

/// Placeholder TTL values from RFC 0002, matching the registry.
const TTL_THRESHOLD: u32 = 518_400;
const DEFAULT_TTL_LEDGERS: u32 = 3_110_400;

/// The part of the attestation registry this contract uses.
#[contractclient(name = "RegistryClient")]
pub trait RegistryInterface {
    fn get_schema(env: Env, id: SchemaId) -> Result<Schema, asterism_types::Error>;
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
}

#[contracttype]
#[derive(Clone)]
enum DataKey {
    Registry,
    Issuer(Address),
    Authorized(SchemaId, Address),
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
