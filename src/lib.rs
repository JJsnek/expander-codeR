//! Library entrypoint for the expander-code prototype.
//!
//! The crate is organized around a recursive encoder that:
//! 1. Projects an input vector down by pairing coordinates.
//! 2. Recursively encodes the projected vector.
//! 3. Applies sparse linear maps derived from sampled bipartite graphs.
//! 4. Appends parity data to form a systematic codeword.
//!
//! The surrounding modules provide field utilities, graph and sparse-matrix
//! generation, experiment harnesses, and a verbose interactive demo.

pub mod demo;
pub mod encoder;
pub mod expander;
pub mod experiment;
pub mod field;
pub mod graph;
pub mod matrix;
pub mod utils;
