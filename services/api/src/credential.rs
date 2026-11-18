//! Credential records and their commitments (RFC 0003, decision 4).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::{
    canonical::canonical_json,
    schema::{is_address, SchemaDocument},
    Error,
};

/// Which registry schema a credential is issued under.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaRef {
    /// Registry contract id (`C...`).
    pub registry: String,
    pub id: u64,
}

/// The off-chain credential kept by the issuer and the holder.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CredentialRecord {
    pub schema: SchemaRef,
    /// The holder's Stellar address.
    pub subject: String,
    pub claims: Map<String, Value>,
    /// 32 random bytes, hex encoded, unique per credential.
    pub salt: String,
}

impl CredentialRecord {
    /// A record with a fresh random salt.
    pub fn new(
        schema: SchemaRef,
        subject: String,
        claims: Map<String, Value>,
    ) -> Result<Self, Error> {
        Ok(Self {
            schema,
            subject,
            claims,
            salt: new_salt()?,
        })
    }

    /// Checks the record against its schema document.
    pub fn validate(&self, document: &SchemaDocument) -> Result<(), Error> {
        if !is_address(&self.subject) {
            return Err(Error::InvalidCredential(
                "subject is not a valid Stellar address".to_string(),
            ));
        }
        if !is_address(&self.schema.registry) {
            return Err(Error::InvalidCredential(
                "schema.registry is not a valid contract id".to_string(),
            ));
        }
        let salt_ok = self.salt.len() == 64 && self.salt.bytes().all(|b| b.is_ascii_hexdigit());
        if !salt_ok {
            return Err(Error::InvalidCredential(
                "salt must be 32 bytes, hex encoded".to_string(),
            ));
        }
        document.validate_claims(&self.claims)
    }

    /// `JCS(record)`: the Merkle leaf bytes for a batch and the preimage of a single commitment.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, Error> {
        let value =
            serde_json::to_value(self).map_err(|e| Error::InvalidCredential(e.to_string()))?;
        Ok(canonical_json(&value)?.into_bytes())
    }

    /// `SHA-256(JCS(record))`: the `data_hash` passed to `issue` for a single credential.
    pub fn commitment(&self) -> Result<[u8; 32], Error> {
        Ok(Sha256::digest(self.canonical_bytes()?).into())
    }
}

fn new_salt() -> Result<String, Error> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|_| Error::Random)?;
    Ok(hex::encode(bytes))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::schema::tests::degree_schema;
    use serde_json::json;

    pub fn registry_id() -> String {
        stellar_strkey::Contract([9; 32]).to_string()
    }

    pub fn holder() -> String {
        stellar_strkey::ed25519::PublicKey([7; 32]).to_string()
    }

    pub fn record() -> CredentialRecord {
        CredentialRecord {
            schema: SchemaRef {
                registry: registry_id(),
                id: 3,
            },
            subject: holder(),
            claims: json!({ "degree": "BSc", "graduated": "2026-07-15" })
                .as_object()
                .cloned()
                .unwrap_or_default(),
            salt: "ab".repeat(32),
        }
    }

    #[test]
    fn valid_record_passes() {
        assert!(record().validate(&degree_schema()).is_ok());
    }

    #[test]
    fn rejects_bad_subject_registry_salt_and_claims() {
        let doc = degree_schema();
        let mut r = record();
        r.subject = "not an address".into();
        assert!(r.validate(&doc).is_err());

        let mut r = record();
        r.schema.registry = "nope".into();
        assert!(r.validate(&doc).is_err());

        let mut r = record();
        r.salt = "abc".into();
        assert!(r.validate(&doc).is_err());

        let mut r = record();
        r.claims.remove("degree");
        assert!(r.validate(&doc).is_err());
    }

    #[test]
    fn commitment_is_deterministic_and_binds_every_part() {
        let base = record();
        assert_eq!(base.commitment().unwrap(), record().commitment().unwrap());

        let mut other_salt = record();
        other_salt.salt = "cd".repeat(32);
        assert_ne!(base.commitment().unwrap(), other_salt.commitment().unwrap());

        let mut other_subject = record();
        other_subject.subject = stellar_strkey::ed25519::PublicKey([8; 32]).to_string();
        assert_ne!(
            base.commitment().unwrap(),
            other_subject.commitment().unwrap()
        );

        let mut other_claim = record();
        other_claim.claims.insert("degree".into(), json!("MSc"));
        assert_ne!(
            base.commitment().unwrap(),
            other_claim.commitment().unwrap()
        );
    }

    #[test]
    fn new_records_get_unique_salts() {
        let a = CredentialRecord::new(record().schema, holder(), Map::new()).unwrap();
        let b = CredentialRecord::new(record().schema, holder(), Map::new()).unwrap();
        assert_eq!(a.salt.len(), 64);
        assert_ne!(a.salt, b.salt);
    }
}
