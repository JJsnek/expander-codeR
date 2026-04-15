//core logic

use crate::matrix::SparseMatrix;
use crate::field::F;
use crate::field::rand_nonzero;

pub struct Layer {
    pub A: SparseMatrix,
    pub B: SparseMatrix,
}

pub fn apply_matrix(x: &[F], m: &SparseMatrix) -> Vec<F> {
    m.apply(x)
}

pub struct EncodingTrace {
    pub layers: Vec<Vec<F>>, // store each layer output
}



pub fn project(x: &[F]) -> Vec<F> {
    let mut res = Vec::with_capacity(x.len() / 2);

    for i in (0..x.len()).step_by(2) {
        let a = rand_nonzero(); // or fixed per layer
        let b = rand_nonzero();

        let v = x[i] * a + x[i + 1] * b;
        res.push(v);
    }

    res
}
//determinism
//For SNARK compatibility later:
//Replace randomness with:
//a, b = hash(layer_id, position)

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


pub fn encode_with_trace(x: Vec<F>, layers: &[Layer]) -> EncodingTrace {
    let mut trace = Vec::new();

    fn helper(x: Vec<F>, layers: &[Layer], trace: &mut Vec<Vec<F>>) -> Vec<F> {
        if layers.is_empty() {
            trace.push(x.clone());
            return x;
        }

        let first = &layers[0];

        let x_proj = project(&x);
        let inner = helper(x_proj, &layers[1..], trace);

        let y = first.A.apply(&inner);
        let parity = first.B.apply(&y);

        let out = [x.clone(), parity].concat();
        trace.push(out.clone());

        out
    }

    helper(x, layers, &mut trace);
    trace.reverse();
    EncodingTrace { layers: trace }
}