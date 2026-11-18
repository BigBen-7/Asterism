use super::*;
use asterism_types::{Commitment, Status};
use attestation_registry::{AttestationRegistry, AttestationRegistryClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    vec, Bytes, Env, String,
};

struct Setup<'a> {
    env: Env,
    client: CredentialsClient<'a>,
    registry: AttestationRegistryClient<'a>,
    creator: Address,
    issuer: Address,
    schema: SchemaId,
}

fn setup<'a>() -> Setup<'a> {
    let env = Env::default();
    env.mock_all_auths();
    let registry_id = env.register(AttestationRegistry, ());
    let registry = AttestationRegistryClient::new(&env, &registry_id);
    let credentials_id = env.register(Credentials, (registry_id,));
    let client = CredentialsClient::new(&env, &credentials_id);

    let creator = Address::generate(&env);
    let issuer = Address::generate(&env);
    let schema = registry.register_schema(
        &creator,
        &String::from_str(&env, "degree"),
        &1,
        &hash(&env, 1),
        &true,
    );
    Setup {
        env,
        client,
        registry,
        creator,
        issuer,
        schema,
    }
}

fn hash(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

fn register(s: &Setup, issuer: &Address) {
    s.client
        .register_issuer(issuer, &String::from_str(&s.env, "Uni"), &hash(&s.env, 2));
}

#[test]
fn constructor_binds_the_registry() {
    let s = setup();
    assert_eq!(s.client.registry(), s.registry.address);
}

// register_issuer / get_issuer

#[test]
fn register_and_read_back_an_issuer() {
    let s = setup();
    s.env.ledger().set_timestamp(42);
    register(&s, &s.issuer);
    let issuer = s.client.get_issuer(&s.issuer);
    assert_eq!(issuer.name, String::from_str(&s.env, "Uni"));
    assert_eq!(issuer.metadata_hash, hash(&s.env, 2));
    assert_eq!(issuer.registered_at, 42);
}

#[test]
fn registering_twice_is_an_error() {
    let s = setup();
    register(&s, &s.issuer);
    let again = s.client.try_register_issuer(
        &s.issuer,
        &String::from_str(&s.env, "Other"),
        &hash(&s.env, 3),
    );
    assert_eq!(again, Err(Ok(Error::AlreadyRegistered)));
    assert_eq!(
        s.client.get_issuer(&s.issuer).name,
        String::from_str(&s.env, "Uni")
    );
}

#[test]
fn register_rejects_a_long_name() {
    let s = setup();
    let long = core::str::from_utf8(&[b'a'; 65]).unwrap();
    let result =
        s.client
            .try_register_issuer(&s.issuer, &String::from_str(&s.env, long), &hash(&s.env, 2));
    assert_eq!(result, Err(Ok(Error::NameTooLong)));
}

#[test]
fn register_requires_issuer_auth() {
    let env = Env::default();
    let registry_id = env.register(AttestationRegistry, ());
    let id = env.register(Credentials, (registry_id,));
    let client = CredentialsClient::new(&env, &id);
    let issuer = Address::generate(&env);
    let result =
        client.try_register_issuer(&issuer, &String::from_str(&env, "Uni"), &hash(&env, 2));
    assert!(result.is_err());
}

#[test]
fn unknown_issuer_is_an_error() {
    let s = setup();
    assert_eq!(
        s.client.try_get_issuer(&s.issuer),
        Err(Ok(Error::NotRegistered))
    );
}

// authorize_issuer / deauthorize_issuer / is_authorized

#[test]
fn creator_can_authorize_a_registered_issuer() {
    let s = setup();
    register(&s, &s.issuer);
    assert!(!s.client.is_authorized(&s.schema, &s.issuer));
    s.client.authorize_issuer(&s.creator, &s.schema, &s.issuer);
    assert!(s.client.is_authorized(&s.schema, &s.issuer));
}

#[test]
fn authorization_is_per_schema() {
    let s = setup();
    register(&s, &s.issuer);
    let other = s.registry.register_schema(
        &s.creator,
        &String::from_str(&s.env, "badge"),
        &1,
        &hash(&s.env, 9),
        &true,
    );
    s.client.authorize_issuer(&s.creator, &s.schema, &s.issuer);
    assert!(s.client.is_authorized(&s.schema, &s.issuer));
    assert!(!s.client.is_authorized(&other, &s.issuer));
}

#[test]
fn only_the_schema_creator_can_authorize() {
    let s = setup();
    register(&s, &s.issuer);
    for caller in [s.issuer.clone(), Address::generate(&s.env)] {
        let result = s.client.try_authorize_issuer(&caller, &s.schema, &s.issuer);
        assert_eq!(result, Err(Ok(Error::NotSchemaCreator)));
    }
    assert!(!s.client.is_authorized(&s.schema, &s.issuer));
}

#[test]
fn authorize_rejects_an_unknown_schema() {
    let s = setup();
    register(&s, &s.issuer);
    let result = s.client.try_authorize_issuer(&s.creator, &99, &s.issuer);
    assert_eq!(result, Err(Ok(Error::SchemaNotFound)));
}

#[test]
fn authorize_rejects_an_unregistered_issuer() {
    let s = setup();
    let result = s
        .client
        .try_authorize_issuer(&s.creator, &s.schema, &s.issuer);
    assert_eq!(result, Err(Ok(Error::NotRegistered)));
}

#[test]
fn authorizing_twice_is_an_error() {
    let s = setup();
    register(&s, &s.issuer);
    s.client.authorize_issuer(&s.creator, &s.schema, &s.issuer);
    let result = s
        .client
        .try_authorize_issuer(&s.creator, &s.schema, &s.issuer);
    assert_eq!(result, Err(Ok(Error::AlreadyAuthorized)));
}

#[test]
fn authorize_requires_creator_auth() {
    let s = setup();
    register(&s, &s.issuer);
    s.env.set_auths(&[]);
    let result = s
        .client
        .try_authorize_issuer(&s.creator, &s.schema, &s.issuer);
    assert!(result.is_err());
    assert!(!s.client.is_authorized(&s.schema, &s.issuer));
}

#[test]
fn creator_can_deauthorize() {
    let s = setup();
    register(&s, &s.issuer);
    s.client.authorize_issuer(&s.creator, &s.schema, &s.issuer);
    s.client
        .deauthorize_issuer(&s.creator, &s.schema, &s.issuer);
    assert!(!s.client.is_authorized(&s.schema, &s.issuer));
    // The issuer stays registered and can be authorized again.
    s.client.get_issuer(&s.issuer);
    s.client.authorize_issuer(&s.creator, &s.schema, &s.issuer);
    assert!(s.client.is_authorized(&s.schema, &s.issuer));
}

#[test]
fn only_the_schema_creator_can_deauthorize() {
    let s = setup();
    register(&s, &s.issuer);
    s.client.authorize_issuer(&s.creator, &s.schema, &s.issuer);
    let result = s
        .client
        .try_deauthorize_issuer(&s.issuer, &s.schema, &s.issuer);
    assert_eq!(result, Err(Ok(Error::NotSchemaCreator)));
    assert!(s.client.is_authorized(&s.schema, &s.issuer));
}

#[test]
fn deauthorizing_a_non_authorized_issuer_is_an_error() {
    let s = setup();
    register(&s, &s.issuer);
    let result = s
        .client
        .try_deauthorize_issuer(&s.creator, &s.schema, &s.issuer);
    assert_eq!(result, Err(Ok(Error::NotAuthorized)));
}

#[test]
fn deauthorize_rejects_an_unknown_schema() {
    let s = setup();
    let result = s.client.try_deauthorize_issuer(&s.creator, &99, &s.issuer);
    assert_eq!(result, Err(Ok(Error::SchemaNotFound)));
}

#[test]
fn unknown_schema_and_unregistered_issuer_are_not_authorized() {
    let s = setup();
    assert!(!s.client.is_authorized(&99, &s.issuer));
    assert!(!s.client.is_authorized(&s.schema, &s.issuer));
}

// issue / revoke / verify

fn authorized_issuer(s: &Setup) {
    register(s, &s.issuer);
    s.client.authorize_issuer(&s.creator, &s.schema, &s.issuer);
}

fn issue(s: &Setup, expires_at: Option<u64>) -> AttestationId {
    let subject = Address::generate(&s.env);
    s.client.issue(
        &s.issuer,
        &s.schema,
        &subject,
        &hash(&s.env, 5),
        &expires_at,
    )
}

#[test]
fn authorized_issuer_can_issue_and_verify() {
    let s = setup();
    authorized_issuer(&s);
    s.env.ledger().set_timestamp(100);
    let subject = Address::generate(&s.env);
    let id = s
        .client
        .issue(&s.issuer, &s.schema, &subject, &hash(&s.env, 5), &Some(500));

    // The registry holds the attestation, with the real issuer recorded.
    let a = s.registry.get(&id);
    assert_eq!(a.issuer, s.issuer);
    assert_eq!(a.subject, Some(subject));
    assert_eq!(a.commitment, Commitment::Single(hash(&s.env, 5)));
    assert_eq!(a.expires_at, Some(500));

    assert_eq!(s.client.status(&id), Status::Valid);
    assert!(s.client.verify(&id));
}

#[test]
fn issue_requires_registration_and_authorization() {
    let s = setup();
    let subject = Address::generate(&s.env);
    let try_issue = |issuer: &Address| {
        s.client
            .try_issue(issuer, &s.schema, &subject, &hash(&s.env, 5), &None)
    };

    // Unknown issuer.
    assert_eq!(try_issue(&s.issuer), Err(Ok(Error::IssuerNotAuthorized)));
    // Registered but not authorized for the schema.
    register(&s, &s.issuer);
    assert_eq!(try_issue(&s.issuer), Err(Ok(Error::IssuerNotAuthorized)));
    // Authorized for a different schema only.
    let other = s.registry.register_schema(
        &s.creator,
        &String::from_str(&s.env, "badge"),
        &1,
        &hash(&s.env, 9),
        &true,
    );
    s.client.authorize_issuer(&s.creator, &other, &s.issuer);
    assert_eq!(try_issue(&s.issuer), Err(Ok(Error::IssuerNotAuthorized)));
}

#[test]
fn issue_requires_issuer_auth() {
    let s = setup();
    authorized_issuer(&s);
    s.env.set_auths(&[]);
    let subject = Address::generate(&s.env);
    let result = s
        .client
        .try_issue(&s.issuer, &s.schema, &subject, &hash(&s.env, 5), &None);
    assert!(result.is_err());
}

#[test]
fn issue_surfaces_registry_errors() {
    let s = setup();
    authorized_issuer(&s);
    s.env.ledger().set_timestamp(1_000);
    let subject = Address::generate(&s.env);
    let result = s.client.try_issue(
        &s.issuer,
        &s.schema,
        &subject,
        &hash(&s.env, 5),
        &Some(1_000),
    );
    assert_eq!(result, Err(Ok(Error::InvalidExpiry)));
}

#[test]
fn credential_expires() {
    let s = setup();
    authorized_issuer(&s);
    s.env.ledger().set_timestamp(100);
    let id = issue(&s, Some(200));
    s.env.ledger().set_timestamp(200);
    assert_eq!(s.client.status(&id), Status::Expired);
    assert!(!s.client.verify(&id));
}

#[test]
fn issuer_can_revoke() {
    let s = setup();
    authorized_issuer(&s);
    let id = issue(&s, None);
    s.client.revoke(&s.issuer, &id);
    assert_eq!(s.client.status(&id), Status::Revoked);
    assert!(!s.client.verify(&id));
    assert_eq!(
        s.client.try_revoke(&s.issuer, &id),
        Err(Ok(Error::AlreadyRevoked))
    );
}

#[test]
fn only_the_issuer_can_revoke() {
    let s = setup();
    authorized_issuer(&s);
    let id = issue(&s, None);
    // Neither a stranger nor the schema creator can revoke.
    for caller in [Address::generate(&s.env), s.creator.clone()] {
        assert_eq!(s.client.try_revoke(&caller, &id), Err(Ok(Error::NotIssuer)));
    }
    assert!(s.client.verify(&id));
}

#[test]
fn non_revocable_schema_rejects_revocation() {
    let s = setup();
    register(&s, &s.issuer);
    let fixed = s.registry.register_schema(
        &s.creator,
        &String::from_str(&s.env, "fixed"),
        &1,
        &hash(&s.env, 9),
        &false,
    );
    s.client.authorize_issuer(&s.creator, &fixed, &s.issuer);
    let subject = Address::generate(&s.env);
    let id = s
        .client
        .issue(&s.issuer, &fixed, &subject, &hash(&s.env, 5), &None);
    assert_eq!(
        s.client.try_revoke(&s.issuer, &id),
        Err(Ok(Error::NotRevocable))
    );
}

#[test]
fn revoke_requires_caller_auth() {
    let s = setup();
    authorized_issuer(&s);
    let id = issue(&s, None);
    s.env.set_auths(&[]);
    assert!(s.client.try_revoke(&s.issuer, &id).is_err());
}

#[test]
fn deauthorized_issuer_keeps_existing_credentials_and_can_still_revoke() {
    let s = setup();
    authorized_issuer(&s);
    let id = issue(&s, None);
    s.client
        .deauthorize_issuer(&s.creator, &s.schema, &s.issuer);

    assert!(s.client.verify(&id));
    let subject = Address::generate(&s.env);
    assert_eq!(
        s.client
            .try_issue(&s.issuer, &s.schema, &subject, &hash(&s.env, 6), &None),
        Err(Ok(Error::IssuerNotAuthorized))
    );
    s.client.revoke(&s.issuer, &id);
    assert_eq!(s.client.status(&id), Status::Revoked);
}

#[test]
fn registry_only_attestations_are_not_credentials() {
    let s = setup();
    let subject = Address::generate(&s.env);
    // Anyone may issue straight on the registry, but that is not a credential.
    let id = s
        .registry
        .issue(&s.issuer, &s.schema, &subject, &hash(&s.env, 5), &None);
    assert_eq!(s.client.try_status(&id), Err(Ok(Error::NotACredential)));
    assert_eq!(s.client.try_verify(&id), Err(Ok(Error::NotACredential)));
    assert_eq!(
        s.client.try_revoke(&s.issuer, &id),
        Err(Ok(Error::NotACredential))
    );
    assert_eq!(s.client.try_status(&99), Err(Ok(Error::NotACredential)));
}

// batch issuance

/// Two-leaf tree hashed the way the registry does it.
fn two_leaf_tree(env: &Env) -> (BytesN<32>, [Bytes; 2], [Vec<BytesN<32>>; 2]) {
    let leaves = [
        Bytes::from_array(env, b"alice"),
        Bytes::from_array(env, b"bob"),
    ];
    let leaf_hash = |leaf: &Bytes| {
        let mut data = Bytes::from_array(env, &[0u8]);
        data.append(leaf);
        env.crypto().sha256(&data).to_bytes()
    };
    let (a, b) = (leaf_hash(&leaves[0]), leaf_hash(&leaves[1]));
    let (left, right) = if a <= b {
        (a.clone(), b.clone())
    } else {
        (b.clone(), a.clone())
    };
    let mut data = Bytes::from_array(env, &[1u8]);
    data.append(&Bytes::from(left));
    data.append(&Bytes::from(right));
    let root = env.crypto().sha256(&data).to_bytes();
    (root, leaves, [vec![env, b], vec![env, a]])
}

#[test]
fn authorized_issuer_can_issue_a_batch_and_members_verify() {
    let s = setup();
    authorized_issuer(&s);
    let (root, leaves, proofs) = two_leaf_tree(&s.env);
    let id = s.client.issue_batch(&s.issuer, &s.schema, &root, &None);

    let a = s.registry.get(&id);
    assert_eq!(a.issuer, s.issuer);
    assert_eq!(a.subject, None);
    assert_eq!(a.commitment, Commitment::Batch(root));

    for i in 0..2 {
        assert!(s.client.verify_inclusion(&id, &leaves[i], &proofs[i]));
    }
    assert!(s.client.verify(&id));
}

#[test]
fn batch_non_member_and_tampered_proof_fail() {
    let s = setup();
    authorized_issuer(&s);
    let (root, leaves, proofs) = two_leaf_tree(&s.env);
    let id = s.client.issue_batch(&s.issuer, &s.schema, &root, &None);

    let outsider = Bytes::from_array(&s.env, b"mallory");
    assert_eq!(
        s.client.try_verify_inclusion(&id, &outsider, &proofs[0]),
        Err(Ok(Error::InvalidProof))
    );
    assert_eq!(
        s.client.try_verify_inclusion(&id, &leaves[0], &proofs[1]),
        Err(Ok(Error::InvalidProof))
    );
}

#[test]
fn batch_requires_authorization_and_is_revoked_as_a_whole() {
    let s = setup();
    let (root, leaves, proofs) = two_leaf_tree(&s.env);
    assert_eq!(
        s.client.try_issue_batch(&s.issuer, &s.schema, &root, &None),
        Err(Ok(Error::IssuerNotAuthorized))
    );

    authorized_issuer(&s);
    let id = s.client.issue_batch(&s.issuer, &s.schema, &root, &None);
    s.client.revoke(&s.issuer, &id);
    assert!(!s.client.verify_inclusion(&id, &leaves[0], &proofs[0]));
    assert_eq!(s.client.status(&id), Status::Revoked);
}

#[test]
fn inclusion_on_a_registry_only_batch_is_not_a_credential() {
    let s = setup();
    let (root, leaves, proofs) = two_leaf_tree(&s.env);
    let id = s.registry.issue_batch(&s.issuer, &s.schema, &root, &None);
    assert_eq!(
        s.client.try_verify_inclusion(&id, &leaves[0], &proofs[0]),
        Err(Ok(Error::NotACredential))
    );
}
