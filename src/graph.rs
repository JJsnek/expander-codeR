//! Random bipartite graph sampling utilities.
//!
//! The current sampler is simple: every left vertex independently chooses `d`
//! right-side neighbors with replacement. That gives a left `d`-regular graph,
//! but does not enforce any stronger expansion properties.

use rand::Rng;

/// Bipartite graph represented by left-to-right adjacency lists.
pub struct Graph {
    pub left_size: usize,
    pub right_size: usize,
    pub adj: Vec<Vec<usize>>,
}

/// Sample a left `d`-regular bipartite graph with replacement.
pub fn sample_d_regular(n: usize, m: usize, d: usize) -> Graph {
    let mut rng = rand::thread_rng();
    let mut adj = vec![vec![]; n];

    for i in 0..n {
        for _ in 0..d {
            let j = rng.gen_range(0..m);
            adj[i].push(j);
        }
    }
    Graph {
        left_size: n,
        right_size: m,
        adj,
    }
}
