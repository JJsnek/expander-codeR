//core logic

use crate::matrix::SparseMatrix;
use crate::field::{F,rand_nonzero,hash_pair};
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

pub fn project(x: &[F], layer_id: usize) -> Vec<F> {
    let mut res = Vec::with_capacity(x.len() / 2);

    for (pos, i) in (0..x.len()).step_by(2).enumerate() {
        let (a, b) = hash_pair(layer_id, pos);

        let v = x[i] * a + x[i + 1] * b;
        res.push(v);
    }

    res
}


//determinism
//For SNARK compatibility later:
//Replace randomness with:
//a, b = hash(layer_id, position)

pub fn encode(x: Vec<F>, layers: &[Layer], layer_id: usize) -> Vec<F> {
    if layers.is_empty() {
        return x;
    }

    let first = &layers[0];

    // 1. projection
    let x_proj = project(&x,layer_id);

    // 2. recursive encoding
    let inner = encode(x_proj, &layers[1..],layer_id + 1);

    // 3. expander (τ_k)
    let y = first.A.apply(&inner);
    let parity = first.B.apply(&y);

    // 4. concatenate (systematic)
    [x, parity].concat()
}



pub fn encode_with_trace(x: Vec<F>, layers: &[Layer]) -> EncodingTrace {
    let mut trace = Vec::new();

    fn helper(
        x: Vec<F>,
        layers: &[Layer],
        trace: &mut Vec<Vec<F>>,
        layer_id: usize,
    ) -> Vec<F> {
        if layers.is_empty() {
            trace.push(x.clone());
            return x;
        }

        let first = &layers[0];

        // ✅ deterministic projection
        let x_proj = project(&x, layer_id);

        // ✅ pass layer_id forward
        let inner = helper(x_proj, &layers[1..], trace, layer_id + 1);

        let y = first.A.apply(&inner);
        let parity = first.B.apply(&y);

        let out = [x.clone(), parity].concat();
        trace.push(out.clone());

        out
    }

    // ✅ start from layer 0
    helper(x, layers, &mut trace, 0);

    trace.reverse();
    EncodingTrace { layers: trace }
}