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
pub fn from_graph_random (g:&Graph) ->SparseMatrix{
    let rows=g.adj.iter().map(|neighbors|{
        neighbors.iter().map(|&j|{
            (j,rand_field())
            }).collect()
        }).collect();
    SparseMatrix{rows}
}

//Spielman-style

pub fn from_graph_nonzero(g:&Graph) -> SparseMatrix{
    let rows=g.adj.iter().map(|neighbors|{
        neighbors.iter().map(|&j|{
            (j,rand_nonzero())
        }).collect()
    }).collect();

    SparseMatrix {rows}
}