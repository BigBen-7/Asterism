#![no_std]
//! Attestation registry: the shared foundation of Asterism.
//!
//! Phase 0 scaffold only. The data model is defined in RFC 0001 (issue 0.1)
//! and the operations (`register_schema`, `issue`, `revoke`, ...) land in
//! issues 0.4 onwards.

use soroban_sdk::{contract, contractimpl};

#[contract]
pub struct AttestationRegistry;

#[contractimpl]
impl AttestationRegistry {
    /// Returns the contract interface version. Placeholder until real operations exist.
    pub fn version() -> u32 {
        asterism_types::INTERFACE_VERSION
    }
}

#[cfg(test)]
mod test;
