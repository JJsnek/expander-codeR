use ark_bn254::Fr;
use ark_ff::{Zero,UniformRand,One,PrimeField};

use rand::thread_rng;

pub type F=Fr;

pub fn rand_field() -> F{
    F::rand(&mut thread_rng())
}

pub fn rand_nonzero() -> F{
    loop{
        let x=rand_field();
        if !x.is_zero(){
            return x;
        }
    }
}

use sha2::{Sha256, Digest};


pub fn hash_to_field(input: &[u8]) -> F {
    let hash = Sha256::digest(input);
    F::from_le_bytes_mod_order(&hash)
}

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