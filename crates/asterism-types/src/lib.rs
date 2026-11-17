#![no_std]
//! Shared Asterism types.
//!
//! The contract-facing core stays `no_std`. Real types arrive with issue 0.3,
//! after RFC 0001 fixes the attestation data model.

/// Version of the registry interface. Bump when the public contract API changes.
pub const INTERFACE_VERSION: u32 = 0;

#[cfg(test)]
mod tests {
    #[test]
    fn interface_version_starts_at_zero() {
        assert_eq!(super::INTERFACE_VERSION, 0);
    }
}
