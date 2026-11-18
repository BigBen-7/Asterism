//! Merkle helpers for batch attestations (RFC 0001, decision 3).
//!
//! Leaves are `SHA-256(0x00 || leaf_bytes)` and interior nodes are
//! `SHA-256(0x01 || left || right)`. Proofs carry no direction bits, so the
//! two children of a node are always ordered bytewise (smaller first). Anyone
//! building a tree for `issue_batch` must hash interior nodes the same way.

use soroban_sdk::{Bytes, BytesN, Env, Vec};

const LEAF_PREFIX: u8 = 0x00;
const NODE_PREFIX: u8 = 0x01;

/// Hashes raw leaf bytes into a leaf node.
pub fn hash_leaf(env: &Env, leaf: &Bytes) -> BytesN<32> {
    let mut data = Bytes::from_array(env, &[LEAF_PREFIX]);
    data.append(leaf);
    env.crypto().sha256(&data).to_bytes()
}

/// Hashes two sibling nodes into their parent, smaller sibling first.
pub fn hash_node(env: &Env, a: &BytesN<32>, b: &BytesN<32>) -> BytesN<32> {
    let (left, right) = if a <= b { (a, b) } else { (b, a) };
    let mut data = Bytes::from_array(env, &[NODE_PREFIX]);
    data.append(&Bytes::from(left.clone()));
    data.append(&Bytes::from(right.clone()));
    env.crypto().sha256(&data).to_bytes()
}

/// Folds `proof` over the hash of `leaf` and returns the resulting root.
/// The loop is bounded by the caller via `MAX_PROOF_LEN`.
pub fn compute_root(env: &Env, leaf: &Bytes, proof: &Vec<BytesN<32>>) -> BytesN<32> {
    let mut node = hash_leaf(env, leaf);
    for sibling in proof.iter() {
        node = hash_node(env, &node, &sibling);
    }
    node
}
