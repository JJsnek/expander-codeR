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
        SamplingMode::Hybrid => unreachable!(), // handled above
    };
    let B=match mode{
        SamplingMode::Random=>from_graph_random(&g2),
        SamplingMode::NonZero=>from_graph_nonzero(&g2),
        SamplingMode::Hybrid => unreachable!(), // handled above
    };

    Layer {A,B}
}
   
