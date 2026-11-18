//! Resource benchmarks for the registry (issue 0.9). Run with `just bench`.
//!
//! Prints a Markdown table of CPU instructions and memory bytes per
//! operation, measured in the Soroban test environment. Ignored by default so
//! `cargo test` stays fast; results go in `docs/benchmarks.md`.

use asterism_types::MAX_PROOF_LEN;
use attestation_registry::{AttestationRegistry, AttestationRegistryClient, RecordKind};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, BytesN, Env, String, Vec,
};

fn h(env: &Env, byte: u8) -> BytesN<32> {
    BytesN::from_array(env, &[byte; 32])
}

/// Runs `f` with a fresh unlimited budget and prints its cost as a table row.
fn measure<T>(env: &Env, name: &str, f: impl FnOnce() -> T) -> T {
    env.cost_estimate().budget().reset_unlimited();
    let out = f();
    let budget = env.cost_estimate().budget();
    println!(
        "| {name} | {} | {} |",
        budget.cpu_instruction_cost(),
        budget.memory_bytes_cost()
    );
    out
}

/// Builds a proof of `depth` dummy sibling hashes.
fn deep_proof(env: &Env, depth: u32) -> Vec<BytesN<32>> {
    let mut proof = Vec::new(env);
    for _ in 0..depth {
        proof.push_back(h(env, 7));
    }
    proof
}

#[test]
#[ignore = "benchmark; run with `just bench`"]
fn registry_operations() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);
    let id = env.register(AttestationRegistry, ());
    let client = AttestationRegistryClient::new(&env, &id);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);

    println!("| Operation | CPU instructions | Memory bytes |");
    println!("| --- | --- | --- |");

    let schema = measure(&env, "register_schema", || {
        client.register_schema(
            &issuer,
            &String::from_str(&env, "degree"),
            &1,
            &h(&env, 1),
            &true,
        )
    });
    measure(&env, "get_schema", || client.get_schema(&schema));
    let att = measure(&env, "issue", || {
        client.issue(&issuer, &schema, &subject, &h(&env, 2), &Some(2_000))
    });
    measure(&env, "get", || client.get(&att));
    measure(&env, "status", || client.status(&att));
    measure(&env, "verify", || client.verify(&att));
    measure(&env, "extend_ttl", || {
        client.extend_ttl(&RecordKind::Attestation, &att)
    });
    measure(&env, "extend_instance_ttl", || client.extend_instance_ttl());
    measure(&env, "revoke", || client.revoke(&issuer, &att));

    let batch = measure(&env, "issue_batch", || {
        client.issue_batch(&issuer, &schema, &h(&env, 3), &None)
    });

    // Inclusion proofs fail with `InvalidProof` here (the root is a dummy),
    // but the hashing work up to the root comparison is the cost being measured.
    let leaf = Bytes::from_array(&env, b"alice");
    for depth in [1, 8, 16, MAX_PROOF_LEN] {
        let proof = deep_proof(&env, depth);
        measure(&env, &format!("verify_inclusion (depth {depth})"), || {
            let _ = client.try_verify_inclusion(&batch, &leaf, &proof);
        });
    }
}
