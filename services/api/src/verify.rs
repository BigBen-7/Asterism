//! Verifying a presented credential against the chain.
//!
//! Steps (RFC 0003, decision 4): validate the schema document and claims,
//! find the attestation, check it belongs to the same registry and schema and
//! that the schema document hashes to the registered `fields_hash`, recompute
//! the commitment from the credential record, and derive the status.

use serde::Serialize;

use crate::{
    chain::{ChainReader, Commitment, OnChainAttestation},
    merkle::{compute_root, Hash, MAX_PROOF_LEN},
    payload::VerifyPayload,
    Error,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Check {
    pub name: &'static str,
    pub passed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct VerifyResult {
    /// True only if every check passed and the status is `valid`.
    pub valid: bool,
    /// `valid`, `expired` or `revoked`; absent if the attestation was not found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<&'static str>,
    pub checks: Vec<Check>,
}

struct Checks(Vec<Check>);

impl Checks {
    fn add(&mut self, name: &'static str, passed: bool, detail: &str) {
        self.0.push(Check {
            name,
            passed,
            detail: (!passed).then(|| detail.to_string()),
        });
    }

    fn finish(self, status: Option<&'static str>) -> VerifyResult {
        let valid = status == Some("valid") && self.0.iter().all(|c| c.passed);
        VerifyResult {
            valid,
            status,
            checks: self.0,
        }
    }
}

/// `now` is Unix seconds, the same clock as the registry's `expires_at`.
pub async fn verify<R: ChainReader>(
    reader: &R,
    registry_id: &str,
    payload: &VerifyPayload,
    now: u64,
) -> Result<VerifyResult, Error> {
    payload.check_version()?;
    let mut checks = Checks(Vec::new());
    let credential = &payload.credential;
    let document = &payload.schema_document;

    let well_formed = document
        .validate()
        .and_then(|()| credential.validate(document));
    checks.add(
        "credential_matches_schema",
        well_formed.is_ok(),
        &well_formed.err().map(|e| e.to_string()).unwrap_or_default(),
    );
    checks.add(
        "registry",
        credential.schema.registry == registry_id,
        "credential names a different registry",
    );

    let Some(attestation) = reader.attestation(payload.attestation_id).await? else {
        checks.add("attestation_found", false, "no such attestation is indexed");
        return Ok(checks.finish(None));
    };
    checks.add("attestation_found", true, "");

    checks.add(
        "schema_id",
        attestation.schema_id == credential.schema.id,
        "attestation was issued under a different schema",
    );
    let doc_hash = document.fields_hash().ok();
    let chain_hash = reader.schema_fields_hash(attestation.schema_id).await?;
    checks.add(
        "schema_document_hash",
        doc_hash.is_some() && doc_hash == chain_hash,
        "schema document does not match the registered fields_hash",
    );
    checks.add(
        "commitment",
        commitment_matches(&attestation, payload),
        "credential does not match the on-chain commitment",
    );

    Ok(checks.finish(Some(status(&attestation, now))))
}

fn status(attestation: &OnChainAttestation, now: u64) -> &'static str {
    if attestation.revoked {
        "revoked"
    } else if attestation.expires_at.is_some_and(|t| t <= now) {
        "expired"
    } else {
        "valid"
    }
}

fn commitment_matches(attestation: &OnChainAttestation, payload: &VerifyPayload) -> bool {
    let credential = &payload.credential;
    match &attestation.commitment {
        Commitment::Single(hash) => {
            attestation.subject.as_deref() == Some(credential.subject.as_str())
                && credential.commitment().is_ok_and(|c| c == *hash)
        }
        Commitment::Batch(root) => {
            let (Ok(leaf), Some(proof)) = (credential.canonical_bytes(), parse_proof(payload))
            else {
                return false;
            };
            compute_root(&leaf, &proof) == *root
        }
    }
}

/// `None` if the proof is malformed or longer than the registry allows.
fn parse_proof(payload: &VerifyPayload) -> Option<Vec<Hash>> {
    let items = payload.proof.as_deref().unwrap_or_default();
    if items.len() > MAX_PROOF_LEN {
        return None;
    }
    items
        .iter()
        .map(|h| hex::decode(h).ok()?.try_into().ok())
        .collect()
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::{
        credential::tests::{holder, record, registry_id},
        merkle,
        payload::tests::payload,
        schema::tests::degree_schema,
    };
    use std::collections::HashMap;

    /// In-memory chain state.
    #[derive(Default)]
    pub struct FakeChain {
        pub attestations: HashMap<u64, OnChainAttestation>,
        pub schemas: HashMap<u64, Hash>,
    }

    impl ChainReader for FakeChain {
        async fn attestation(&self, id: u64) -> Result<Option<OnChainAttestation>, Error> {
            Ok(self.attestations.get(&id).cloned())
        }
        async fn schema_fields_hash(&self, schema_id: u64) -> Result<Option<Hash>, Error> {
            Ok(self.schemas.get(&schema_id).copied())
        }
    }

    /// A chain holding a valid single attestation (id 12) for `record()`.
    pub fn chain() -> FakeChain {
        let mut chain = FakeChain::default();
        chain
            .schemas
            .insert(3, degree_schema().fields_hash().unwrap());
        chain.attestations.insert(
            12,
            OnChainAttestation {
                id: 12,
                schema_id: 3,
                issuer: "GISSUER".into(),
                subject: Some(holder()),
                commitment: Commitment::Single(record().commitment().unwrap()),
                expires_at: Some(2_000),
                revoked: false,
            },
        );
        chain
    }

    fn failed(result: &VerifyResult) -> Vec<&'static str> {
        result
            .checks
            .iter()
            .filter(|c| !c.passed)
            .map(|c| c.name)
            .collect()
    }

    async fn run(chain: &FakeChain, p: &VerifyPayload, now: u64) -> VerifyResult {
        verify(chain, &registry_id(), p, now).await.unwrap()
    }

    #[tokio::test]
    async fn a_genuine_single_credential_is_valid() {
        let r = run(&chain(), &payload(), 1_000).await;
        assert!(r.valid, "{r:?}");
        assert_eq!(r.status, Some("valid"));
    }

    #[tokio::test]
    async fn expired_and_revoked_credentials_are_not_valid() {
        let c = chain();
        let r = run(&c, &payload(), 2_000).await;
        assert!(!r.valid);
        assert_eq!(r.status, Some("expired"));
        assert!(failed(&r).is_empty());

        let mut c = chain();
        if let Some(a) = c.attestations.get_mut(&12) {
            a.revoked = true;
        }
        let r = run(&c, &payload(), 1_000).await;
        assert!(!r.valid);
        assert_eq!(r.status, Some("revoked"));
    }

    #[tokio::test]
    async fn unknown_attestation_is_reported() {
        let mut p = payload();
        p.attestation_id = 99;
        let r = run(&chain(), &p, 1_000).await;
        assert!(!r.valid);
        assert_eq!(r.status, None);
        assert_eq!(failed(&r), vec!["attestation_found"]);
    }

    #[tokio::test]
    async fn altered_claims_salt_or_subject_fail_the_commitment() {
        let c = chain();
        let mut p = payload();
        p.credential.claims.insert("degree".into(), "PhD".into());
        assert_eq!(failed(&run(&c, &p, 1_000).await), vec!["commitment"]);

        let mut p = payload();
        p.credential.salt = "cd".repeat(32);
        assert_eq!(failed(&run(&c, &p, 1_000).await), vec!["commitment"]);

        let mut p = payload();
        p.credential.subject = stellar_strkey::ed25519::PublicKey([8; 32]).to_string();
        assert!(failed(&run(&c, &p, 1_000).await).contains(&"commitment"));
    }

    #[tokio::test]
    async fn wrong_schema_registry_or_document_is_detected() {
        let c = chain();
        let mut p = payload();
        p.credential.schema.id = 4;
        assert!(failed(&run(&c, &p, 1_000).await).contains(&"schema_id"));

        let mut p = payload();
        p.credential.schema.registry = stellar_strkey::Contract([1; 32]).to_string();
        assert!(failed(&run(&c, &p, 1_000).await).contains(&"registry"));

        let mut p = payload();
        p.schema_document.version = 2;
        assert!(failed(&run(&c, &p, 1_000).await).contains(&"schema_document_hash"));
    }

    #[tokio::test]
    async fn invalid_claims_fail_the_schema_check() {
        let mut p = payload();
        p.credential.claims.remove("degree");
        let r = run(&chain(), &p, 1_000).await;
        assert!(failed(&r).contains(&"credential_matches_schema"));
    }

    /// A chain holding batch attestation 20 over three credentials.
    fn batch_chain() -> (FakeChain, Vec<VerifyPayload>) {
        let records: Vec<_> = (0..3)
            .map(|i| {
                let mut r = record();
                r.salt = format!("{i:02x}").repeat(32);
                r
            })
            .collect();
        let leaves: Vec<Vec<u8>> = records
            .iter()
            .map(|r| r.canonical_bytes().unwrap())
            .collect();
        let tree = merkle::build(&leaves).unwrap();

        let mut chain = FakeChain::default();
        chain
            .schemas
            .insert(3, degree_schema().fields_hash().unwrap());
        chain.attestations.insert(
            20,
            OnChainAttestation {
                id: 20,
                schema_id: 3,
                issuer: "GISSUER".into(),
                subject: None,
                commitment: Commitment::Batch(tree.root),
                expires_at: None,
                revoked: false,
            },
        );
        let payloads = records
            .into_iter()
            .zip(tree.proofs)
            .map(|(credential, proof)| VerifyPayload {
                v: 1,
                attestation_id: 20,
                schema_document: degree_schema(),
                credential,
                proof: Some(proof.iter().map(hex::encode).collect()),
            })
            .collect();
        (chain, payloads)
    }

    #[tokio::test]
    async fn every_member_of_a_batch_verifies() {
        let (chain, payloads) = batch_chain();
        for p in &payloads {
            let r = run(&chain, p, 1_000_000).await;
            assert!(r.valid, "{r:?}");
        }
    }

    #[tokio::test]
    async fn non_members_and_bad_proofs_fail_a_batch() {
        let (chain, payloads) = batch_chain();

        let mut outsider = payloads[0].clone();
        outsider
            .credential
            .claims
            .insert("degree".into(), "PhD".into());
        assert_eq!(failed(&run(&chain, &outsider, 1).await), vec!["commitment"]);

        let mut wrong_proof = payloads[0].clone();
        wrong_proof.proof = payloads[1].proof.clone();
        assert_eq!(
            failed(&run(&chain, &wrong_proof, 1).await),
            vec!["commitment"]
        );

        let mut malformed = payloads[0].clone();
        malformed.proof = Some(vec!["zz".to_string()]);
        assert_eq!(
            failed(&run(&chain, &malformed, 1).await),
            vec!["commitment"]
        );

        let mut too_long = payloads[0].clone();
        too_long.proof = Some(vec!["00".repeat(32); MAX_PROOF_LEN + 1]);
        assert_eq!(failed(&run(&chain, &too_long, 1).await), vec!["commitment"]);
    }
}
