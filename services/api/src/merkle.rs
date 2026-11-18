//! Merkle trees with the same hashing as the registry (`contracts/attestation-registry/src/merkle.rs`).
//!
//! Leaves are `SHA-256(0x00 || leaf_bytes)`, interior nodes are
//! `SHA-256(0x01 || left || right)` with the two children ordered bytewise
//! (smaller first), so proofs need no direction bits. An unpaired node at any
//! level is carried up unchanged. A proof is the list of sibling hashes from
//! the leaf to the root, skipping levels where the node had no sibling.

use sha2::{Digest, Sha256};

pub type Hash = [u8; 32];

/// Maximum proof length the registry accepts (`MAX_PROOF_LEN`).
pub const MAX_PROOF_LEN: usize = 32;

pub fn hash_leaf(leaf: &[u8]) -> Hash {
    let mut h = Sha256::new();
    h.update([0x00]);
    h.update(leaf);
    h.finalize().into()
}

pub fn hash_node(a: &Hash, b: &Hash) -> Hash {
    let (left, right) = if a <= b { (a, b) } else { (b, a) };
    let mut h = Sha256::new();
    h.update([0x01]);
    h.update(left);
    h.update(right);
    h.finalize().into()
}

/// A built tree: the root and one proof per input leaf, in input order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    pub root: Hash,
    pub proofs: Vec<Vec<Hash>>,
}

/// Builds a tree over `leaves`. Returns `None` for an empty input.
pub fn build(leaves: &[Vec<u8>]) -> Option<Tree> {
    if leaves.is_empty() {
        return None;
    }
    let mut level: Vec<Hash> = leaves.iter().map(|l| hash_leaf(l)).collect();
    // `positions[i]` is the index of input leaf i within the current level.
    let mut positions: Vec<usize> = (0..leaves.len()).collect();
    let mut proofs: Vec<Vec<Hash>> = vec![Vec::new(); leaves.len()];

    while level.len() > 1 {
        for (leaf, pos) in positions.iter().enumerate() {
            let sibling = pos ^ 1;
            if let (Some(hash), Some(proof)) = (level.get(sibling), proofs.get_mut(leaf)) {
                proof.push(*hash);
            }
        }
        level = level
            .chunks(2)
            .map(|pair| match pair {
                [a, b] => hash_node(a, b),
                [a] => *a,
                _ => [0; 32],
            })
            .collect();
        for pos in positions.iter_mut() {
            *pos /= 2;
        }
    }
    let root = level.first().copied()?;
    Some(Tree { root, proofs })
}

/// Folds `proof` over the hash of `leaf` and returns the resulting root.
pub fn compute_root(leaf: &[u8], proof: &[Hash]) -> Hash {
    proof
        .iter()
        .fold(hash_leaf(leaf), |node, sibling| hash_node(&node, sibling))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn leaves(n: usize) -> Vec<Vec<u8>> {
        (0..n).map(|i| format!("leaf-{i}").into_bytes()).collect()
    }

    #[test]
    fn empty_input_has_no_tree() {
        assert_eq!(build(&[]), None);
    }

    #[test]
    fn single_leaf_root_is_its_hash_with_an_empty_proof() {
        let tree = build(&leaves(1)).unwrap();
        assert_eq!(tree.root, hash_leaf(b"leaf-0"));
        assert!(tree.proofs[0].is_empty());
    }

    #[test]
    fn every_leaf_proves_membership_for_many_sizes() {
        for n in 1..=17 {
            let input = leaves(n);
            let tree = build(&input).unwrap();
            for (leaf, proof) in input.iter().zip(&tree.proofs) {
                assert_eq!(compute_root(leaf, proof), tree.root, "n={n}");
                assert!(proof.len() <= MAX_PROOF_LEN);
            }
        }
    }

    #[test]
    fn non_members_and_tampered_proofs_do_not_match() {
        let tree = build(&leaves(5)).unwrap();
        assert_ne!(compute_root(b"mallory", &tree.proofs[0]), tree.root);
        assert_ne!(compute_root(b"leaf-0", &tree.proofs[1]), tree.root);
        let mut bad = tree.proofs[0].clone();
        bad[0][0] ^= 1;
        assert_ne!(compute_root(b"leaf-0", &bad), tree.root);
    }

    #[test]
    fn node_hash_is_order_independent() {
        let a = hash_leaf(b"a");
        let b = hash_leaf(b"b");
        assert_eq!(hash_node(&a, &b), hash_node(&b, &a));
    }
}
