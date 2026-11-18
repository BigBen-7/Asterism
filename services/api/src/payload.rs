//! The verification payload a holder presents, and its link / QR form.
//!
//! A payload carries everything a verifier needs besides the chain: the
//! credential record, its schema document, where the attestation is
//! (`attestation_id`) and, for batch credentials, the Merkle proof.
//!
//! Link form: `<base>/verify#<payload>` where `<payload>` is the payload's
//! canonical JSON (RFC 8785) encoded as base64url without padding. The
//! payload is in the URL fragment, so it is never sent to a server by a
//! browser. A QR code simply encodes the link string.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};

use crate::{
    canonical::canonical_json, credential::CredentialRecord, schema::SchemaDocument, Error,
};

/// Payload format version.
pub const VERSION: u32 = 1;

/// Upper bound on a decoded link payload, to keep verification cheap.
const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VerifyPayload {
    /// Payload format version; must be [`VERSION`].
    pub v: u32,
    /// Registry attestation id (the credential, or the batch it belongs to).
    pub attestation_id: u64,
    pub schema_document: SchemaDocument,
    pub credential: CredentialRecord,
    /// Merkle proof for batch credentials: sibling hashes, hex, leaf to root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof: Option<Vec<String>>,
}

impl VerifyPayload {
    pub fn check_version(&self) -> Result<(), Error> {
        if self.v == VERSION {
            Ok(())
        } else {
            Err(Error::InvalidPayload(format!(
                "unsupported payload version {}",
                self.v
            )))
        }
    }

    /// Canonical JSON bytes of the payload.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, Error> {
        let value = serde_json::to_value(self).map_err(|e| Error::InvalidPayload(e.to_string()))?;
        Ok(canonical_json(&value)?.into_bytes())
    }

    /// The shareable link (and QR content) for this payload.
    pub fn to_link(&self, base_url: &str) -> Result<String, Error> {
        let encoded = URL_SAFE_NO_PAD.encode(self.canonical_bytes()?);
        Ok(format!(
            "{}/verify#{encoded}",
            base_url.trim_end_matches('/')
        ))
    }

    /// Parses a link produced by [`VerifyPayload::to_link`]. Only the fragment is read.
    pub fn from_link(link: &str) -> Result<Self, Error> {
        let (_, fragment) = link
            .split_once('#')
            .ok_or_else(|| Error::InvalidPayload("link has no payload fragment".to_string()))?;
        let bytes = URL_SAFE_NO_PAD
            .decode(fragment)
            .map_err(|_| Error::InvalidPayload("payload is not valid base64url".to_string()))?;
        if bytes.len() > MAX_PAYLOAD_BYTES {
            return Err(Error::InvalidPayload("payload is too large".to_string()));
        }
        let payload: Self =
            serde_json::from_slice(&bytes).map_err(|e| Error::InvalidPayload(e.to_string()))?;
        payload.check_version()?;
        Ok(payload)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{credential::tests::record, schema::tests::degree_schema};

    pub fn payload() -> VerifyPayload {
        VerifyPayload {
            v: VERSION,
            attestation_id: 12,
            schema_document: degree_schema(),
            credential: record(),
            proof: None,
        }
    }

    #[test]
    fn link_roundtrip() {
        let p = payload();
        let link = p.to_link("https://verify.example/").unwrap();
        assert!(link.starts_with("https://verify.example/verify#"));
        assert_eq!(VerifyPayload::from_link(&link).unwrap(), p);
    }

    #[test]
    fn link_roundtrip_with_proof() {
        let mut p = payload();
        p.proof = Some(vec!["00".repeat(32), "ff".repeat(32)]);
        let link = p.to_link("http://localhost:3000").unwrap();
        assert_eq!(VerifyPayload::from_link(&link).unwrap(), p);
    }

    #[test]
    fn the_same_payload_always_gives_the_same_link() {
        assert_eq!(
            payload().to_link("https://v.example").unwrap(),
            payload().to_link("https://v.example").unwrap()
        );
    }

    #[test]
    fn bad_links_are_rejected() {
        assert!(VerifyPayload::from_link("https://v.example/verify").is_err());
        assert!(VerifyPayload::from_link("https://v.example/verify#%%%").is_err());
        let not_json = URL_SAFE_NO_PAD.encode(b"hello");
        assert!(VerifyPayload::from_link(&format!("x#{not_json}")).is_err());
    }

    #[test]
    fn unsupported_versions_are_rejected() {
        let mut p = payload();
        p.v = 2;
        let link = p.to_link("https://v.example").unwrap();
        assert!(VerifyPayload::from_link(&link).is_err());
    }
}
