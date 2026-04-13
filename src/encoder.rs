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



pub fn encode_recursive(x: Vec<F>, layers: &[Layer]) -> Vec<F> {
    if layers.is_empty() {
        return x;
    }

    let first = &layers[0];

    // Apply A then B FIRST
    let y = first.A.apply(&x);
    let z = first.B.apply(&y);

    // Then recurse
    encode_recursive(z, &layers[1..])
}