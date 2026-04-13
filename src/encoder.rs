//core logic

use crate::matrix::SparseMatrix;
use crate::field::F;

pub struct Layer {
    pub A: SparseMatrix,
    pub B: SparseMatrix,
}

pub fn apply_matrix(x: &[F], m: &SparseMatrix) -> Vec<F> {
    m.apply(x)
}



use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn project(x: &[F]) -> Vec<F> {
    x.iter().step_by(2).cloned().collect()
}


pub fn encode(x: Vec<F>, layers: &[Layer]) -> Vec<F> {
    if layers.is_empty() {
        return x;
    }

    let first = &layers[0];

    // 1. projection
    let x_proj = project(&x);

    // 2. recursive encoding
    let inner = encode(x_proj, &layers[1..]);

    // 3. expander (τ_k)
    let y = first.A.apply(&inner);
    let parity = first.B.apply(&y);

    // 4. concatenate (systematic)
    [x, parity].concat()
}