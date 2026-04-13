use crate::graph::sample_d_regular;
use crate::matrix::{from_graph_random,from_graph_nonzero, SparseMatrix};
use crate::experiment::ExperimentConfig;
use crate::encoder::Layer;


#[derive(Clone, Copy, Debug)]
pub enum SamplingMode{
    Random,
    NonZero,
    Hybrid,
}


//1 layer

   
pub fn build_layer(
    n: usize,
    m: usize,
    d: usize,
    mode: SamplingMode,
) -> Layer {
    let g1 = sample_d_regular(n, m, d);
    let g2 = sample_d_regular(m, n, d);

    match mode {
        SamplingMode::Random => {
            let A = from_graph_random(&g1);
            let B = from_graph_random(&g2);
            Layer { A, B }
        }

        SamplingMode::NonZero => {
            let A = from_graph_nonzero(&g1);
            let B = from_graph_nonzero(&g2);
            Layer { A, B }
        }

        SamplingMode::Hybrid => {
            let A = from_graph_nonzero(&g1);  // structured
            let B = from_graph_random(&g2);   // random
            Layer { A, B }
        }
    }
}