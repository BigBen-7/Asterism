#![no_std]
//! Shared Asterism types (RFC 0001).
//!
//! `no_std` so contracts can depend on it directly. Services use the same
//! types through `soroban-sdk`'s host-side build.

use soroban_sdk::{contracterror, contracttype, Address, BytesN, String};

/// Version of the registry interface. Bump when the public contract API changes.
pub const INTERFACE_VERSION: u32 = 0;

/// Maximum schema name length in bytes.
pub const MAX_NAME_LEN: u32 = 64;

/// Maximum Merkle proof length (levels), keeping verification loops bounded.
pub const MAX_PROOF_LEN: u32 = 32;

/// Schema identifier, assigned from a monotonically increasing counter.
pub type SchemaId = u64;

/// Attestation identifier, assigned from a monotonically increasing counter.
pub type AttestationId = u64;

/// A registered schema. Immutable once registered.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Schema {
    pub name: String,
    pub version: u32,
    /// SHA-256 of the off-chain schema definition.
    pub fields_hash: BytesN<32>,
    /// Fixed at registration; non-revocable schemas reject `revoke`.
    pub revocable: bool,
    pub creator: Address,
}

/// What an attestation commits to. Payloads stay off-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Commitment {
    /// SHA-256 hash of a single claim's data.
    Single(BytesN<32>),
    /// Merkle root over many claims.
    Batch(BytesN<32>),
}

/// An on-chain statement made by `issuer` under a schema.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attestation {
    pub schema_id: SchemaId,
    pub issuer: Address,
    /// `Some` for single attestations, `None` for batches.
    pub subject: Option<Address>,
    pub commitment: Commitment,
    /// Ledger timestamp at issuance.
    pub issued_at: u64,
    pub expires_at: Option<u64>,
    pub revoked_at: Option<u64>,
}

/// Derived validity of an attestation at a point in time.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Valid,
    Expired,
    Revoked,
}

impl Attestation {
    /// Revoked wins over expired; expiry is inclusive (`expires_at <= now`).
    pub fn status(&self, now: u64) -> Status {
        if self.revoked_at.is_some() {
            return Status::Revoked;
        }
        match self.expires_at {
            Some(t) if t <= now => Status::Expired,
            _ => Status::Valid,
        }
    }
}

/// Registry errors. Codes are stable and append-only.
#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    SchemaNotFound = 1,
    AttestationNotFound = 2,
    NameTooLong = 3,
    InvalidExpiry = 4,
    NotRevocable = 5,
    AlreadyRevoked = 6,
    NotIssuer = 7,
    InvalidProof = 8,
    ProofTooLong = 9,
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    fn sample(env: &Env, expires_at: Option<u64>, revoked_at: Option<u64>) -> Attestation {
        Attestation {
            schema_id: 1,
            issuer: Address::generate(env),
            subject: Some(Address::generate(env)),
            commitment: Commitment::Single(BytesN::from_array(env, &[7; 32])),
            issued_at: 100,
            expires_at,
            revoked_at,
        }
    }

    #[test]
    fn interface_version_starts_at_zero() {
        assert_eq!(INTERFACE_VERSION, 0);
    }

    #[test]
    fn valid_without_expiry() {
        let env = Env::default();
        assert_eq!(sample(&env, None, None).status(u64::MAX), Status::Valid);
    }

    #[test]
    fn valid_before_expiry_expired_at_expiry() {
        let env = Env::default();
        let a = sample(&env, Some(200), None);
        assert_eq!(a.status(199), Status::Valid);
        assert_eq!(a.status(200), Status::Expired);
        assert_eq!(a.status(201), Status::Expired);
    }

    #[test]
    fn revoked_wins_over_expired() {
        let env = Env::default();
        let a = sample(&env, Some(200), Some(150));
        assert_eq!(a.status(300), Status::Revoked);
        assert_eq!(a.status(0), Status::Revoked);
    }

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(Error::SchemaNotFound as u32, 1);
        assert_eq!(Error::ProofTooLong as u32, 9);
    }

    #[test]
    fn types_roundtrip_through_val() {
        use soroban_sdk::{IntoVal, TryFromVal, Val};
        let env = Env::default();
        let a = sample(&env, Some(5), None);
        let v: Val = a.clone().into_val(&env);
        assert_eq!(Attestation::try_from_val(&env, &v).unwrap(), a);
        let c = Commitment::Batch(BytesN::from_array(&env, &[1; 32]));
        let v: Val = c.clone().into_val(&env);
        assert_eq!(Commitment::try_from_val(&env, &v).unwrap(), c);
    }
}
