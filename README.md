# expander-codeR

`expander-codeR` is a Rust prototype for experimenting with a recursive expander-style coding construction over the BN254 scalar field.

The project is not a polished library yet. It is a research scaffold that:

- samples sparse bipartite graphs
- turns those graphs into sparse linear maps
- recursively encodes vectors by projection plus parity generation
- compares several coefficient-sampling strategies
- writes experiment summaries to CSV files

## Repository Layout

- [src/main.rs](./src/main.rs): binary entrypoint, demo runner, and experiment sweeps
- [src/lib.rs](./src/lib.rs): crate module wiring
- [src/field.rs](./src/field.rs): BN254 field helpers and hash-to-field projection coefficients
- [src/graph.rs](./src/graph.rs): simple left `d`-regular bipartite graph sampler
- [src/matrix.rs](./src/matrix.rs): sparse matrix representation and graph-to-matrix conversion
- [src/encoder.rs](./src/encoder.rs): recursive encoder and trace recording
- [src/expander.rs](./src/expander.rs): layer construction and sampling modes
- [src/experiment.rs](./src/experiment.rs): batch experiment harness and CSV export
- [src/demo.rs](./src/demo.rs): interactive trace-printing demo

## Core Idea

Each recursive layer does the following:

1. Pair adjacent coordinates of the current vector.
2. Compress each pair into one field element using deterministic hash-derived coefficients.
3. Recursively encode the smaller vector.
4. Apply sparse matrix `A` to the recursive output.
5. Apply sparse matrix `B` to the result of `A`.
6. Emit a systematic codeword `[x || parity]`.

The sparse matrices are generated from sampled bipartite graphs. The code compares three coefficient assignment modes:

- `Random`: random field weights on both matrices
- `NonZero`: random nonzero field weights on both matrices
- `Hybrid`: nonzero weights on `A`, random weights on `B`

## Running

Build the crate:

```bash
cargo check
```

Run the executable:

```bash
cargo run
```

The program first asks for an input size `n` for the interactive demo. After the demo, it runs several longer experiment sweeps and writes CSV files into the repository root:

- `results.csv`
- `results_n_scaling.csv`
- `results_cost_model.csv`

## Current Behavior

The demo prints:

- a random input vector
- the recursive encoding trace
- the innermost output before and after corruption
- a simple sampling-based detection result

The batch experiment harness measures:

- average encoding time
- a failure count derived from the current local consistency test

## Improvements Made

The following verifier improvements were implemented from Miloš's suggestions:

- the experiment harness now corrupts a trace layer and verifies that corrupted trace instead of generating a detached corrupted vector and then checking the pristine trace
- the demo now prints and verifies the same corrupted innermost trace layer, so the displayed object matches the object being checked
- the codebase now includes a deterministic full-trace verifier, which checks every parity row in every layer and serves as the reference notion of trace consistency
- regression tests now cover both clean traces and deliberately corrupted traces

## Important Limitations

This repository compiles and runs, but it is still prototype-grade overall.

- the graph sampler is lightweight and does not prove or enforce formal expansion guarantees
- the randomized `local_test` used in batch runs is still a spot-check verifier rather than an exhaustive one
- Some configuration fields, such as `alpha_num` and `alpha_den`, are stored in configs mainly for bookkeeping and sweep structure rather than deep internal use.

## Output Interpretation

The CSV exports are useful for rough comparisons between modes and parameter settings, especially for runtime trends. They now reflect verification over explicitly corrupted traces, but they still should not be interpreted as rigorous coding-theory results without stronger graph and soundness analysis.
