//! Finite-field helpers for the prototype.
//!
//! All arithmetic is performed in the BN254 scalar field. The module also
//! provides hash-to-field utilities used to derive deterministic projection
//! coefficients from `(layer_id, position)`.

use ark_bn254::Fr;
use ark_ff::{One, PrimeField, UniformRand, Zero};

use rand::thread_rng;

pub type F = Fr;

/// Sample a uniformly random field element.
pub fn rand_field() -> F {
    F::rand(&mut thread_rng())
}

/// Sample a uniformly random nonzero field element.
pub fn rand_nonzero() -> F {
    loop {
        let x = rand_field();
        if !x.is_zero() {
            return x;
        }
    }
}

use sha2::{Digest, Sha256};

/// Hash arbitrary bytes into a field element.
pub fn hash_to_field(input: &[u8]) -> F {
    let hash = Sha256::digest(input);
    F::from_le_bytes_mod_order(&hash)
}

/// Derive the two projection coefficients for one paired position.
pub fn hash_pair(layer_id: usize, position: usize) -> (F, F) {
    let mut base = Vec::new();
    base.extend_from_slice(&layer_id.to_le_bytes());
    base.extend_from_slice(&position.to_le_bytes());

    let mut input_a = base.clone();
    input_a.push(0);

    let mut input_b = base;
    input_b.push(1);

    let mut a = hash_to_field(&input_a);
    let mut b = hash_to_field(&input_b);

    // avoid zero if needed
    if a.is_zero() {
        a = F::one();
    }
    if b.is_zero() {
        b = F::one();
    }

    (a, b)
}
