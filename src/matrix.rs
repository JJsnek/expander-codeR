//sparse matrix

use ark_ff::Zero;
use crate::graph::Graph;
use crate::field::{F,rand_field,rand_nonzero};

pub struct SparseMatrix{
    pub rows: Vec<Vec<(usize,F)>>,
}

impl SparseMatrix{
    pub fn apply(&self,x: &[F])->Vec<F>{
        let mut result = vec![F::zero();self.rows.len()];

        for (i,row) in self.rows.iter().enumerate(){
            let mut acc=F::zero();

            for(col,val) in row{
                acc+=*val *x[*col];
            }
            result[i]=acc;
        }
        result
    }
}

//Brakedown-style


pub fn from_graph_random(g: &Graph) -> SparseMatrix {
    let mut rows = vec![Vec::new(); g.right_size];

    for (i, neighbors) in g.adj.iter().enumerate() {
        for &j in neighbors {
            rows[j].push((i, rand_field()));
        }
    }

    SparseMatrix { rows }
}

//Spielman-style
pub fn from_graph_nonzero(g: &Graph) -> SparseMatrix {
    let mut rows = vec![Vec::new(); g.right_size];

    for (i, neighbors) in g.adj.iter().enumerate() {
        for &j in neighbors {
            rows[j].push((i, rand_nonzero()));
        }
    }

    SparseMatrix { rows }
}
