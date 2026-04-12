use ark_bn254::Fr;
use ark_ff::{Zero,UniformRand};
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