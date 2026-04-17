//! Construction of encoding layers from sampled expander-like graphs.
//!
//! The code compares three coefficient-sampling modes while keeping the graph
//! sampling procedure fixed.

use crate::encoder::Layer;
use crate::graph::sample_d_regular;
use crate::matrix::{from_graph_nonzero, from_graph_random};

#[derive(Clone, Copy, Debug)]
/// How matrix coefficients are sampled for each layer.
pub enum SamplingMode {
    Random,
    NonZero,
    Hybrid,
}

/// Build a single recursive layer.
pub fn build_layer(n: usize, m: usize, d: usize, mode: SamplingMode) -> Layer {
    let g1 = sample_d_regular(n, m, d);
    let g2 = sample_d_regular(m, n, d);

    match mode {
        SamplingMode::Random => {
            // Fully random coefficients on both sparse maps.
            let A = from_graph_random(&g1);
            let B = from_graph_random(&g2);
            Layer { A, B }
        }

        SamplingMode::NonZero => {
            // Nonzero-only coefficients on both sparse maps.
            let A = from_graph_nonzero(&g1);
            let B = from_graph_nonzero(&g2);
            Layer { A, B }
        }

        SamplingMode::Hybrid => {
            // Mixed variant: nonzero coefficients for A, random coefficients for B.
            let A = from_graph_nonzero(&g1); // structured
            let B = from_graph_random(&g2); // random
            Layer { A, B }
        }
    }
}
