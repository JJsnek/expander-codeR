//! Core recursive encoding logic.
//!
//! A layer consists of two sparse linear maps `A` and `B`. Given an input:
//! 1. The vector is projected to half size using deterministic hash-derived
//!    coefficients.
//! 2. The projected vector is recursively encoded.
//! 3. The inner codeword is mapped through `A`, then `B`, to produce parity.
//! 4. The final output is `[x || parity]`, so the code is systematic.

use crate::field::{F, hash_pair};
use crate::matrix::SparseMatrix;

/// One recursive encoding layer.
pub struct Layer {
    pub A: SparseMatrix,
    pub B: SparseMatrix,
}

/// Convenience wrapper for sparse-matrix application.
pub fn apply_matrix(x: &[F], m: &SparseMatrix) -> Vec<F> {
    m.apply(x)
}

/// Captures the codeword produced at each recursive depth.
pub struct EncodingTrace {
    pub layers: Vec<Vec<F>>, // store each layer output
}

/// Project adjacent coordinate pairs into one coordinate per pair.
pub fn project(x: &[F], layer_id: usize) -> Vec<F> {
    let mut res = Vec::with_capacity(x.len() / 2);

    for (pos, i) in (0..x.len()).step_by(2).enumerate() {
        let (a, b) = hash_pair(layer_id, pos);

        let v = x[i] * a + x[i + 1] * b;
        res.push(v);
    }

    res
}
/// Recursively encode a vector using the provided layers.
pub fn encode(x: Vec<F>, layers: &[Layer], layer_id: usize) -> Vec<F> {
    if layers.is_empty() {
        return x;
    }

    let first = &layers[0];

    // First shrink the vector deterministically.
    let x_proj = project(&x, layer_id);

    // Then recursively encode the projected vector.
    let inner = encode(x_proj, &layers[1..], layer_id + 1);

    // The sparse maps derive parity information from the inner codeword.
    let y = first.A.apply(&inner);
    let parity = first.B.apply(&y);

    // The code is systematic, so the original input stays visible up front.
    [x, parity].concat()
}

/// Encode a vector while storing the codeword at every recursive layer.
pub fn encode_with_trace(x: Vec<F>, layers: &[Layer]) -> EncodingTrace {
    let mut trace = Vec::new();

    fn helper(x: Vec<F>, layers: &[Layer], trace: &mut Vec<Vec<F>>, layer_id: usize) -> Vec<F> {
        if layers.is_empty() {
            trace.push(x.clone());
            return x;
        }

        let first = &layers[0];

        // Every layer starts by collapsing coordinate pairs using hash-derived
        // coefficients specific to this layer and pair position.
        let x_proj = project(&x, layer_id);

        // Recurse to the next layer on the projected vector.
        let inner = helper(x_proj, &layers[1..], trace, layer_id + 1);

        let y = first.A.apply(&inner);
        let parity = first.B.apply(&y);

        // Record the systematic codeword produced at this layer.
        let out = [x.clone(), parity].concat();
        trace.push(out.clone());

        out
    }

    // Start the recursion at the outermost layer.
    helper(x, layers, &mut trace, 0);

    // The helper records outputs inside-out, so reverse them for a natural
    // outermost-to-innermost ordering.
    trace.reverse();
    EncodingTrace { layers: trace }
}
