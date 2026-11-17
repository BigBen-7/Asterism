use super::*;
use soroban_sdk::Env;

#[test]
fn version_matches_types_crate() {
    let env = Env::default();
    let id = env.register(AttestationRegistry, ());
    let client = AttestationRegistryClient::new(&env, &id);
    assert_eq!(client.version(), asterism_types::INTERFACE_VERSION);
}
