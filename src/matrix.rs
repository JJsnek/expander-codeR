//! Sparse matrix helpers.
//!
//! Sparse matrices are built from sampled bipartite graphs by turning edges into
//! weighted matrix entries. The row-oriented storage is convenient for applying
//! the matrix to a dense vector.

use crate::field::{F, rand_field, rand_nonzero};
use crate::graph::Graph;
use ark_ff::Zero;

/// Sparse matrix stored as rows of `(column, value)` entries.
#[derive(Clone)]
pub struct SparseMatrix {
    pub rows: Vec<Vec<(usize, F)>>,
}

impl SparseMatrix {
    /// Multiply the sparse matrix by a dense vector.
    pub fn apply(&self, x: &[F]) -> Vec<F> {
        let mut result = vec![F::zero(); self.rows.len()];

        for (i, row) in self.rows.iter().enumerate() {
            let mut acc = F::zero();

            for (col, val) in row {
                //
                assert!(
                    *col < x.len(),
                    "Matrix access out of bounds: col {} >= {}",
                    col,
                    x.len()
                );

                acc += *val * x[*col];
            }
            result[i] = acc;
        }
        result
    }
}

/// Build a sparse matrix whose edge weights are fully random field elements.
pub fn from_graph_random(g: &Graph) -> SparseMatrix {
    let mut rows = vec![Vec::new(); g.right_size];

    for (i, neighbors) in g.adj.iter().enumerate() {
        for &j in neighbors {
            rows[j].push((i, rand_field()));
        }
    }

    SparseMatrix { rows }
}

/// Build a sparse matrix whose edge weights are random nonzero field elements.
pub fn from_graph_nonzero(g: &Graph) -> SparseMatrix {
    let mut rows = vec![Vec::new(); g.right_size];

    for (i, neighbors) in g.adj.iter().enumerate() {
        for &j in neighbors {
            rows[j].push((i, rand_nonzero()));
        }
    }

    SparseMatrix { rows }
}
