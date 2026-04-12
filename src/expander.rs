use crate::graph::sample_d_regular;
use crate::matrix::{from_graph_random,from_graph_nonzero, SparseMatrix};
use crate::encoder::Layer;
use crate::field::F;

pub enum SamplingMode{
    Random,
    NonZero,
}

pub fn build_layer(
    n:usize,
    m:usize,
    d:usize,
    mode:SamplingMode,
)->Layer {
    let g1=sample_d_regular(n, m, d);
    let g2=sample_d_regular(m, n, d);

    let A= match mode{
        SamplingMode::Random=>from_graph_random(&g1),
        SamplingMode::NonZero=>from_graph_random(&g1),
    };
    let B=match mode{
        SamplingMode::Random=>from_graph_random(&g2),
        SamplingMode::NonZero=>from_graph_nonzero(&g2),
    };

    Layer {A,B}
}
   
