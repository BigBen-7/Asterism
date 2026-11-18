use super::*;
use crate::merkle::{hash_leaf, hash_node};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Bytes, Env, String,
};

struct Setup<'a> {
    env: Env,
    client: AttestationRegistryClient<'a>,
    issuer: Address,
    subject: Address,
}

fn setup<'a>() -> Setup<'a> {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);
    let id = env.register(AttestationRegistry, ());
    let client = AttestationRegistryClient::new(&env, &id);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    Setup {
        env,
        client,
        issuer,
        subject,
    }
}

fn hash(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

fn register(s: &Setup, revocable: bool) -> SchemaId {
    s.client.register_schema(
        &s.issuer,
        &String::from_str(&s.env, "degree"),
        &1,
        &hash(&s.env, 1),
        &revocable,
    )
}

#[test]
fn version_matches_types_crate() {
    let s = setup();
    assert_eq!(s.client.version(), asterism_types::INTERFACE_VERSION);
}

// register_schema / get_schema

#[test]
fn register_schema_and_read_back() {
    let s = setup();
    let id = register(&s, true);
    assert_eq!(id, 1);
    let schema = s.client.get_schema(&id);
    assert_eq!(schema.name, String::from_str(&s.env, "degree"));
    assert_eq!(schema.version, 1);
    assert_eq!(schema.fields_hash, hash(&s.env, 1));
    assert!(schema.revocable);
    assert_eq!(schema.creator, s.issuer);
}

#[test]
fn schema_ids_are_sequential_and_unique() {
    let s = setup();
    assert_eq!(register(&s, true), 1);
    assert_eq!(register(&s, false), 2);
    assert_eq!(register(&s, true), 3);
    assert!(!s.client.get_schema(&2).revocable);
}

#[test]
fn unknown_schema_is_an_error() {
    let s = setup();
    assert_eq!(s.client.try_get_schema(&99), Err(Ok(Error::SchemaNotFound)));
}

#[test]
fn register_schema_rejects_long_name() {
    let s = setup();
    let long = core::str::from_utf8(&[b'a'; 65]).unwrap();
    let result = s.client.try_register_schema(
        &s.issuer,
        &String::from_str(&s.env, long),
        &1,
        &hash(&s.env, 1),
        &true,
    );
    assert_eq!(result, Err(Ok(Error::NameTooLong)));
}

#[test]
fn register_schema_accepts_max_length_name() {
    let s = setup();
    let max = core::str::from_utf8(&[b'a'; 64]).unwrap();
    let id = s.client.register_schema(
        &s.issuer,
        &String::from_str(&s.env, max),
        &1,
        &hash(&s.env, 1),
        &true,
    );
    assert_eq!(id, 1);
}

#[test]
fn register_schema_requires_creator_auth() {
    // No `mock_all_auths`: the creator's authorization is missing.
    let env = Env::default();
    let id = env.register(AttestationRegistry, ());
    let client = AttestationRegistryClient::new(&env, &id);
    let creator = Address::generate(&env);
    let result = client.try_register_schema(
        &creator,
        &String::from_str(&env, "degree"),
        &1,
        &hash(&env, 1),
        &true,
    );
    assert!(result.is_err());
}

// issue / get

#[test]
fn issue_and_get() {
    let s = setup();
    let schema = register(&s, true);
    let id = s.client.issue(
        &s.issuer,
        &schema,
        &s.subject,
        &hash(&s.env, 2),
        &Some(5_000),
    );
    assert_eq!(id, 1);

    let a = s.client.get(&id);
    assert_eq!(a.schema_id, schema);
    assert_eq!(a.issuer, s.issuer);
    assert_eq!(a.subject, Some(s.subject.clone()));
    assert_eq!(a.commitment, Commitment::Single(hash(&s.env, 2)));
    assert_eq!(a.issued_at, 1_000);
    assert_eq!(a.expires_at, Some(5_000));
    assert_eq!(a.revoked_at, None);
    assert_eq!(s.client.status(&id), Status::Valid);
    assert!(s.client.verify(&id));
}

#[test]
fn issue_rejects_unknown_schema() {
    let s = setup();
    let result = s
        .client
        .try_issue(&s.issuer, &7, &s.subject, &hash(&s.env, 2), &None);
    assert_eq!(result, Err(Ok(Error::SchemaNotFound)));
}

#[test]
fn issue_rejects_expiry_not_in_future() {
    let s = setup();
    let schema = register(&s, true);
    for t in [0, 999, 1_000] {
        let result = s
            .client
            .try_issue(&s.issuer, &schema, &s.subject, &hash(&s.env, 2), &Some(t));
        assert_eq!(result, Err(Ok(Error::InvalidExpiry)));
    }
}

#[test]
fn issue_requires_issuer_auth() {
    let env = Env::default();
    let id = env.register(AttestationRegistry, ());
    let client = AttestationRegistryClient::new(&env, &id);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    // Registering needs auth too, so give it only for this one call.
    env.mock_all_auths();
    let schema = client.register_schema(
        &issuer,
        &String::from_str(&env, "degree"),
        &1,
        &hash(&env, 1),
        &true,
    );
    env.set_auths(&[]);
    let result = client.try_issue(&issuer, &schema, &subject, &hash(&env, 2), &None);
    assert!(result.is_err());
}

#[test]
fn get_unknown_attestation_is_an_error() {
    let s = setup();
    assert_eq!(s.client.try_get(&1), Err(Ok(Error::AttestationNotFound)));
    assert_eq!(s.client.try_status(&1), Err(Ok(Error::AttestationNotFound)));
}

#[test]
fn attestation_expires_but_stays_readable() {
    let s = setup();
    let schema = register(&s, true);
    let id = s.client.issue(
        &s.issuer,
        &schema,
        &s.subject,
        &hash(&s.env, 2),
        &Some(2_000),
    );
    s.env.ledger().set_timestamp(1_999);
    assert_eq!(s.client.status(&id), Status::Valid);
    s.env.ledger().set_timestamp(2_000);
    assert_eq!(s.client.status(&id), Status::Expired);
    assert!(!s.client.verify(&id));
    assert_eq!(s.client.get(&id).expires_at, Some(2_000));
}

// revoke

#[test]
fn issuer_can_revoke() {
    let s = setup();
    let schema = register(&s, true);
    let id = s
        .client
        .issue(&s.issuer, &schema, &s.subject, &hash(&s.env, 2), &None);
    s.env.ledger().set_timestamp(1_500);
    s.client.revoke(&s.issuer, &id);

    assert_eq!(s.client.get(&id).revoked_at, Some(1_500));
    assert_eq!(s.client.status(&id), Status::Revoked);
    assert!(!s.client.verify(&id));
}

#[test]
fn revoked_wins_over_expired() {
    let s = setup();
    let schema = register(&s, true);
    let id = s.client.issue(
        &s.issuer,
        &schema,
        &s.subject,
        &hash(&s.env, 2),
        &Some(2_000),
    );
    s.client.revoke(&s.issuer, &id);
    s.env.ledger().set_timestamp(3_000);
    assert_eq!(s.client.status(&id), Status::Revoked);
}

#[test]
fn only_the_issuer_can_revoke() {
    let s = setup();
    let schema = register(&s, true);
    let id = s
        .client
        .issue(&s.issuer, &schema, &s.subject, &hash(&s.env, 2), &None);
    for caller in [s.subject.clone(), Address::generate(&s.env)] {
        assert_eq!(s.client.try_revoke(&caller, &id), Err(Ok(Error::NotIssuer)));
    }
    assert_eq!(s.client.status(&id), Status::Valid);
}

#[test]
fn revoke_requires_caller_auth() {
    let env = Env::default();
    let id = env.register(AttestationRegistry, ());
    let client = AttestationRegistryClient::new(&env, &id);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    env.mock_all_auths();
    let schema = client.register_schema(
        &issuer,
        &String::from_str(&env, "degree"),
        &1,
        &hash(&env, 1),
        &true,
    );
    let att = client.issue(&issuer, &schema, &subject, &hash(&env, 2), &None);
    env.set_auths(&[]);
    assert!(client.try_revoke(&issuer, &att).is_err());
}

#[test]
fn non_revocable_schema_rejects_revocation() {
    let s = setup();
    let schema = register(&s, false);
    let id = s
        .client
        .issue(&s.issuer, &schema, &s.subject, &hash(&s.env, 2), &None);
    assert_eq!(
        s.client.try_revoke(&s.issuer, &id),
        Err(Ok(Error::NotRevocable))
    );
    assert_eq!(s.client.status(&id), Status::Valid);
}

#[test]
fn revoking_twice_is_an_error() {
    let s = setup();
    let schema = register(&s, true);
    let id = s
        .client
        .issue(&s.issuer, &schema, &s.subject, &hash(&s.env, 2), &None);
    s.client.revoke(&s.issuer, &id);
    assert_eq!(
        s.client.try_revoke(&s.issuer, &id),
        Err(Ok(Error::AlreadyRevoked))
    );
}

#[test]
fn revoke_unknown_attestation_is_an_error() {
    let s = setup();
    assert_eq!(
        s.client.try_revoke(&s.issuer, &1),
        Err(Ok(Error::AttestationNotFound))
    );
}

// issue_batch / verify_inclusion

struct Tree {
    leaves: [Bytes; 4],
    proofs: [Vec<BytesN<32>>; 4],
    root: BytesN<32>,
}

/// Four-leaf tree built with the same hashing as the contract.
fn tree(env: &Env) -> Tree {
    let leaves = [
        Bytes::from_array(env, b"alice"),
        Bytes::from_array(env, b"bob"),
        Bytes::from_array(env, b"carol"),
        Bytes::from_array(env, b"dave"),
    ];
    let h: [BytesN<32>; 4] = [
        hash_leaf(env, &leaves[0]),
        hash_leaf(env, &leaves[1]),
        hash_leaf(env, &leaves[2]),
        hash_leaf(env, &leaves[3]),
    ];
    let n01 = hash_node(env, &h[0], &h[1]);
    let n23 = hash_node(env, &h[2], &h[3]);
    let root = hash_node(env, &n01, &n23);
    let proofs = [
        vec![env, h[1].clone(), n23.clone()],
        vec![env, h[0].clone(), n23.clone()],
        vec![env, h[3].clone(), n01.clone()],
        vec![env, h[2].clone(), n01.clone()],
    ];
    Tree {
        leaves,
        proofs,
        root,
    }
}

#[test]
fn issue_batch_stores_root() {
    let s = setup();
    let schema = register(&s, true);
    let t = tree(&s.env);
    let id = s
        .client
        .issue_batch(&s.issuer, &schema, &t.root, &Some(9_000));
    let a = s.client.get(&id);
    assert_eq!(a.subject, None);
    assert_eq!(a.commitment, Commitment::Batch(t.root.clone()));
    assert_eq!(a.expires_at, Some(9_000));
    assert_eq!(s.client.status(&id), Status::Valid);
}

#[test]
fn issue_batch_rejects_unknown_schema_and_bad_expiry() {
    let s = setup();
    let t = tree(&s.env);
    assert_eq!(
        s.client.try_issue_batch(&s.issuer, &3, &t.root, &None),
        Err(Ok(Error::SchemaNotFound))
    );
    let schema = register(&s, true);
    assert_eq!(
        s.client
            .try_issue_batch(&s.issuer, &schema, &t.root, &Some(1_000)),
        Err(Ok(Error::InvalidExpiry))
    );
}

#[test]
fn issue_batch_requires_issuer_auth() {
    let env = Env::default();
    let id = env.register(AttestationRegistry, ());
    let client = AttestationRegistryClient::new(&env, &id);
    let issuer = Address::generate(&env);
    env.mock_all_auths();
    let schema = client.register_schema(
        &issuer,
        &String::from_str(&env, "degree"),
        &1,
        &hash(&env, 1),
        &true,
    );
    env.set_auths(&[]);
    assert!(client
        .try_issue_batch(&issuer, &schema, &hash(&env, 3), &None)
        .is_err());
}

#[test]
fn every_member_verifies() {
    let s = setup();
    let schema = register(&s, true);
    let t = tree(&s.env);
    let id = s.client.issue_batch(&s.issuer, &schema, &t.root, &None);
    for i in 0..4 {
        assert!(s.client.verify_inclusion(&id, &t.leaves[i], &t.proofs[i]));
    }
}

#[test]
fn non_member_fails() {
    let s = setup();
    let schema = register(&s, true);
    let t = tree(&s.env);
    let id = s.client.issue_batch(&s.issuer, &schema, &t.root, &None);
    let outsider = Bytes::from_array(&s.env, b"mallory");
    assert_eq!(
        s.client.try_verify_inclusion(&id, &outsider, &t.proofs[0]),
        Err(Ok(Error::InvalidProof))
    );
}

#[test]
fn tampered_proof_fails() {
    let s = setup();
    let schema = register(&s, true);
    let t = tree(&s.env);
    let id = s.client.issue_batch(&s.issuer, &schema, &t.root, &None);

    // Wrong sibling.
    let bad = vec![&s.env, hash(&s.env, 9), t.proofs[0].get_unchecked(1)];
    assert_eq!(
        s.client.try_verify_inclusion(&id, &t.leaves[0], &bad),
        Err(Ok(Error::InvalidProof))
    );
    // Truncated proof.
    let short = vec![&s.env, t.proofs[0].get_unchecked(0)];
    assert_eq!(
        s.client.try_verify_inclusion(&id, &t.leaves[0], &short),
        Err(Ok(Error::InvalidProof))
    );
    // Proof belonging to another leaf.
    assert_eq!(
        s.client
            .try_verify_inclusion(&id, &t.leaves[0], &t.proofs[2]),
        Err(Ok(Error::InvalidProof))
    );
    // Empty proof.
    assert_eq!(
        s.client
            .try_verify_inclusion(&id, &t.leaves[0], &Vec::new(&s.env)),
        Err(Ok(Error::InvalidProof))
    );
}

#[test]
fn proof_longer_than_max_is_rejected() {
    let s = setup();
    let schema = register(&s, true);
    let t = tree(&s.env);
    let id = s.client.issue_batch(&s.issuer, &schema, &t.root, &None);
    let mut long = Vec::new(&s.env);
    for _ in 0..=MAX_PROOF_LEN {
        long.push_back(hash(&s.env, 4));
    }
    assert_eq!(
        s.client.try_verify_inclusion(&id, &t.leaves[0], &long),
        Err(Ok(Error::ProofTooLong))
    );
}

#[test]
fn inclusion_is_false_once_batch_is_revoked_or_expired() {
    let s = setup();
    let schema = register(&s, true);
    let t = tree(&s.env);
    let expiring = s
        .client
        .issue_batch(&s.issuer, &schema, &t.root, &Some(2_000));
    let revoked = s.client.issue_batch(&s.issuer, &schema, &t.root, &None);
    s.client.revoke(&s.issuer, &revoked);

    assert!(!s
        .client
        .verify_inclusion(&revoked, &t.leaves[1], &t.proofs[1]));
    s.env.ledger().set_timestamp(2_000);
    assert!(!s
        .client
        .verify_inclusion(&expiring, &t.leaves[1], &t.proofs[1]));
}

#[test]
fn inclusion_on_single_attestation_is_invalid() {
    let s = setup();
    let schema = register(&s, true);
    let t = tree(&s.env);
    let id = s
        .client
        .issue(&s.issuer, &schema, &s.subject, &hash(&s.env, 2), &None);
    assert_eq!(
        s.client
            .try_verify_inclusion(&id, &t.leaves[0], &t.proofs[0]),
        Err(Ok(Error::InvalidProof))
    );
}

#[test]
fn inclusion_on_unknown_attestation_is_an_error() {
    let s = setup();
    let t = tree(&s.env);
    assert_eq!(
        s.client
            .try_verify_inclusion(&5, &t.leaves[0], &t.proofs[0]),
        Err(Ok(Error::AttestationNotFound))
    );
}

#[test]
fn batch_revocation_counts_as_whole_batch() {
    let s = setup();
    let schema = register(&s, false);
    let t = tree(&s.env);
    let id = s.client.issue_batch(&s.issuer, &schema, &t.root, &None);
    assert_eq!(
        s.client.try_revoke(&s.issuer, &id),
        Err(Ok(Error::NotRevocable))
    );
}
