//graph sampling

use rand::Rng;

pub struct Graph{
    pub left_size: usize,
    pub right_size: usize,
    pub adj: Vec<Vec<usize>>,
}

pub fn sample_d_regular (n:usize,m:usize,d:usize) -> Graph{

    let mut rng=rand::thread_rng();
    let mut adj=vec![vec![];n];

    for i in 0..n{
        for _ in 0..d{
            let j=rng.gen_range(0..m);
            adj[i].push(j);
        }
    }
    Graph{
        left_size:n,
        right_size:m,
        adj,
    }
}