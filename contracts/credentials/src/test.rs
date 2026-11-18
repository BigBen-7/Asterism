use super::*;
use attestation_registry::{AttestationRegistry, AttestationRegistryClient};
use soroban_sdk::{testutils::Address as _, Env, String};

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
