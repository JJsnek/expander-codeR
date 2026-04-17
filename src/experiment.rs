//! Experiment harnesses for the recursive expander-code prototype.
//!
//! This module builds layer stacks, samples inputs, runs encoding benchmarks,
//! performs the current local consistency check, and exports aggregate data to
//! CSV for later analysis.
//!
//! The verification helpers below include a Miloš-requested improvement: instead
//! of corrupting a codeword and then accidentally verifying the original trace,
//! the experiment path now constructs a corrupted trace and verifies that exact
//! object.

use crate::encoder::{EncodingTrace, Layer, encode_with_trace};
use crate::expander::{SamplingMode, build_layer};
use crate::field::{F, rand_field};
use ark_ff::Zero;
use rand::seq::SliceRandom;
use rand::thread_rng;

/// Full configuration for one experiment batch.
pub struct ExperimentConfig {
    pub n: usize,
    pub alpha_num: usize,
    pub alpha_den: usize,
    pub d: usize,
    pub layers: usize,
    pub trials: usize,
    pub weight: usize,
    pub mode: SamplingMode,
}

/// Build the recursive layer stack for an experiment.
///
/// Each layer halves the logical input length before the next recursive step, so
/// `n` must remain divisible by two throughout the requested depth.
pub fn build_layers(cfg: &ExperimentConfig) -> Vec<Layer> {
    let mut layers = Vec::new();
    let mut current_n = cfg.n;

    for _ in 0..cfg.layers {
        assert!(
            current_n % 2 == 0,
            "n must be divisible by 2 at every layer"
        );

        let projected_n = current_n / 2;

        let layer = build_layer(
            projected_n, // input size after projection
            projected_n, // keep the layer square for this prototype
            cfg.d,
            cfg.mode,
        );

        layers.push(layer);
        current_n = projected_n;
    }

    layers
}

use ark_ff::UniformRand;

/// Sample a dense random input vector.
pub fn random_vector(n: usize) -> Vec<F> {
    let mut rng = thread_rng();
    (0..n).map(|_| F::rand(&mut rng)).collect()
}

/// Sample a sparse vector with exactly `weight` random nonzero positions.
pub fn random_sparse_vector(n: usize, weight: usize) -> Vec<F> {
    let mut v = vec![F::zero(); n];

    let mut indices: Vec<usize> = (0..n).collect();
    indices.shuffle(&mut thread_rng());

    for &i in indices.iter().take(weight) {
        v[i] = rand_field();
    }

    v
}

/// Overwrite `num_errors` random positions with fresh random field elements.
///
/// The corruption routine does not ensure the new value differs from the old
/// one, so a small fraction of selected positions can remain unchanged by
/// chance.
pub fn corrupt(v: &mut [F], num_errors: usize) {
    use rand::seq::SliceRandom;

    let mut rng = rand::thread_rng();
    let mut indices: Vec<_> = (0..v.len()).collect();
    indices.shuffle(&mut rng);

    for &i in indices.iter().take(num_errors) {
        v[i] = rand_field();
    }
}

/// Deterministically corrupt specific positions in a vector.
///
/// This helper exists mainly to support verifier tests and trace-splicing logic.
/// When a sampled replacement accidentally equals the current value, the helper
/// falls back to adding one so the requested position is guaranteed to change.
pub fn corrupt_positions(v: &mut [F], positions: &[usize]) {
    use ark_ff::One;

    for &i in positions {
        assert!(i < v.len(), "corruption index out of bounds");

        let original = v[i];
        let mut replacement = rand_field();

        if replacement == original {
            replacement += F::one();
        }

        v[i] = replacement;
    }
}

/// Check whether every coordinate is zero.
pub fn is_zero_vector(v: &[F]) -> bool {
    v.iter().all(|x| x.is_zero())
}

/// Reconstruct the systematic input length at each recorded trace layer.
///
/// The encoder halves the logical input length before recursing. That means the
/// innermost trace entry is the base-case vector, and every outer layer's
/// systematic prefix length is exactly twice the next inner layer length.
fn trace_input_lengths(trace: &EncodingTrace, layers: &[Layer]) -> Vec<usize> {
    assert_eq!(
        trace.layers.len(),
        layers.len() + 1,
        "trace length must equal number of layers plus the base case"
    );

    let mut input_lengths = vec![0; layers.len()];
    let mut current_input_len = trace.layers.last().map(Vec::len).unwrap_or(0);

    for layer_idx in (0..layers.len()).rev() {
        current_input_len *= 2;
        input_lengths[layer_idx] = current_input_len;
    }

    input_lengths
}

/// Recompute one parity row for one layer of a trace.
///
/// The function expands the sampled row as `B[row] * (A * current)` where
/// `current` is the next inner trace layer and `next` is the systematic codeword
/// at the current layer.
fn recompute_parity_row(
    trace: &EncodingTrace,
    layers: &[Layer],
    layer_idx: usize,
    row: usize,
) -> F {
    let current = &trace.layers[layer_idx + 1];
    let layer = &layers[layer_idx];

    let mut acc = F::zero();

    for (col, val) in &layer.B.rows[row] {
        let mut inner_acc = F::zero();

        for (inner_col, inner_val) in &layer.A.rows[*col] {
            inner_acc += *inner_val * current[*inner_col];
        }

        acc += *val * inner_acc;
    }

    acc
}

/// Verify a trace against a caller-provided list of rows to check per layer.
///
/// This is the most explicit verifier in the module and underpins both the
/// randomized spot-check verifier and the deterministic full verifier used by
/// tests.
pub fn verify_trace_rows(
    trace: &EncodingTrace,
    layers: &[Layer],
    rows_to_check: &[Vec<usize>],
) -> bool {
    assert_eq!(
        rows_to_check.len(),
        layers.len(),
        "must provide one row list per encoding layer"
    );

    let input_lengths = trace_input_lengths(trace, layers);

    for (layer_idx, rows) in rows_to_check.iter().enumerate() {
        let next = &trace.layers[layer_idx];
        let x_len = input_lengths[layer_idx];
        assert!(
            next.len() >= x_len,
            "trace layer shorter than its inferred systematic prefix"
        );
        let parity = &next[x_len..];

        for &row in rows {
            assert!(row < parity.len(), "requested row out of bounds");

            if recompute_parity_row(trace, layers, layer_idx, row) != parity[row] {
                return false;
            }
        }
    }

    true
}

/// Exhaustively verify every parity row in every layer of a trace.
///
/// This was added as part of Miloš's verifier-tightening suggestion so the
/// codebase has one deterministic, unambiguous notion of
/// "the trace is consistent".
pub fn verify_trace_fully(trace: &EncodingTrace, layers: &[Layer]) -> bool {
    let rows_to_check: Vec<Vec<usize>> = layers
        .iter()
        .map(|layer| (0..layer.B.rows.len()).collect())
        .collect();

    verify_trace_rows(trace, layers, &rows_to_check)
}

/// Clone a trace and replace one layer with a corrupted version.
///
/// The caller identifies which trace layer to mutate and which coordinates of
/// that layer should change. This makes corruption explicit and reproducible,
/// which is useful both for tests and for end-to-end experiments.
pub fn corrupted_trace_at_positions(
    trace: &EncodingTrace,
    layer_idx: usize,
    positions: &[usize],
) -> EncodingTrace {
    assert!(layer_idx < trace.layers.len(), "trace layer out of bounds");

    let mut corrupted_trace = trace.clone();
    corrupt_positions(&mut corrupted_trace.layers[layer_idx], positions);
    corrupted_trace
}

/// Clone a trace and randomly corrupt one chosen layer.
///
/// Suggestion from Miloš: the experiment should verify the actually corrupted
/// object, not silently keep checking the pristine trace. This helper exists to
/// make that workflow the default.
pub fn corrupt_trace_layer(
    trace: &EncodingTrace,
    layer_idx: usize,
    num_errors: usize,
) -> EncodingTrace {
    assert!(layer_idx < trace.layers.len(), "trace layer out of bounds");

    let mut corrupted_trace = trace.clone();
    corrupt(&mut corrupted_trace.layers[layer_idx], num_errors);
    corrupted_trace
}

/// Run a randomized local parity-consistency test over a recorded trace.
///
/// For each layer, the routine samples `num_checks` rows and recomputes the
/// parity contribution implied by `A` and `B`. It returns `false` as soon as one
/// mismatch is found.
pub fn local_test(trace: &EncodingTrace, layers: &[Layer], num_checks: usize) -> bool {
    use rand::Rng;

    let mut rng = rand::thread_rng();
    let rows_to_check: Vec<Vec<usize>> = layers
        .iter()
        .map(|layer| {
            (0..num_checks)
                .map(|_| rng.gen_range(0..layer.B.rows.len()))
                .collect()
        })
        .collect();

    verify_trace_rows(trace, layers, &rows_to_check)
}

use std::time::Instant;

/// Aggregate results from one experiment batch.
pub struct ExperimentResult {
    pub mode: SamplingMode,
    pub n: usize,
    pub d: usize,
    pub layers: usize,
    pub weight: usize,
    pub trials: usize,
    pub failures: usize,
    pub avg_time_ms: f64,
}

/// Run one batch of trials and summarize the observed runtime and failures.
///
/// The current implementation measures encoding time and then verifies a trace
/// whose innermost layer has been explicitly corrupted. This is another
/// Miloš-requested improvement over the earlier prototype, which generated a
/// corrupted value but accidentally kept verifying the pristine trace.
pub fn run_experiment(cfg: &ExperimentConfig) -> ExperimentResult {
    let layers = build_layers(cfg);

    let mut failures = 0;
    let mut total_time = 0.0;

    for _ in 0..cfg.trials {
        let x = random_sparse_vector(cfg.n, cfg.weight);
        let start = Instant::now();
        let trace = encode_with_trace(x, &layers);
        let elapsed = start.elapsed().as_secs_f64();

        total_time += elapsed;

        // Corrupt the innermost trace layer because every outer parity relation
        // ultimately depends on it through the recursive construction.
        let corrupted_trace = corrupt_trace_layer(&trace, trace.layers.len() - 1, cfg.weight);
        let detected = !local_test(&corrupted_trace, &layers, 5);

        if !detected {
            failures += 1;
        }
    }

    ExperimentResult {
        mode: cfg.mode,
        n: cfg.n,
        d: cfg.d,
        layers: cfg.layers,
        weight: cfg.weight,
        trials: cfg.trials,
        failures,
        avg_time_ms: (total_time / cfg.trials as f64) * 1000.0,
    }
}

use std::fs::File;
use std::io::Write;

/// Write experiment summaries to a CSV file.
pub fn write_csv(results: &[ExperimentResult], filename: &str) {
    let mut file = File::create(filename).unwrap();

    writeln!(
        file,
        "mode,n,d,layers,weight,trials,failures,failure_rate,avg_time_ms"
    )
    .unwrap();

    for r in results {
        let failure_rate = r.failures as f64 / r.trials as f64;

        let mode_str = match r.mode {
            SamplingMode::Random => "brakedown",
            SamplingMode::NonZero => "spielman",
            SamplingMode::Hybrid => "hybrid",
        };

        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{}",
            mode_str,
            r.n,
            r.d,
            r.layers,
            r.weight,
            r.trials,
            r.failures,
            failure_rate,
            r.avg_time_ms
        )
        .unwrap();
    }
}

/// Heuristic parameter guess used by the current experiments.
///
/// This is not derived from a formal optimizer in the codebase; it is a simple
/// scaling rule that tries to map `(alpha, rho, delta)` to a plausible
/// `(c, d)` pair for the benchmark sweeps in `main`.
pub fn guess_cd(alpha: f64, rho: f64, delta: f64) -> (usize, usize) {
    // Delta is treated as the primary driver for the baseline tradeoff.
    let base_c = (4.0 + delta * 80.0) as usize;
    let base_d = (18.0 - delta * 80.0) as usize;

    // Then tilt the balance according to the alpha/rho ratio.
    let k = alpha / rho;

    let d = ((base_d as f64) * k).round().max(6.0) as usize;
    let c = ((base_c as f64) * (1.0 / k)).round().max(2.0) as usize;

    (c, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expander::SamplingMode;

    fn small_cfg() -> ExperimentConfig {
        ExperimentConfig {
            n: 16,
            alpha_num: 1,
            alpha_den: 3,
            d: 5,
            layers: 2,
            trials: 1,
            weight: 2,
            mode: SamplingMode::Hybrid,
        }
    }

    #[test]
    fn clean_trace_passes_full_verification() {
        let cfg = small_cfg();
        let layers = build_layers(&cfg);
        let trace = encode_with_trace(random_vector(cfg.n), &layers);

        assert!(verify_trace_fully(&trace, &layers));
    }

    #[test]
    fn corrupting_innermost_trace_breaks_full_verification() {
        let cfg = small_cfg();
        let layers = build_layers(&cfg);
        let trace = encode_with_trace(random_vector(cfg.n), &layers);
        let corrupted_trace = corrupted_trace_at_positions(&trace, trace.layers.len() - 1, &[0]);

        assert!(!verify_trace_fully(&corrupted_trace, &layers));
    }

    #[test]
    fn corrupting_parity_coordinates_breaks_full_verification() {
        let cfg = small_cfg();
        let layers = build_layers(&cfg);
        let trace = encode_with_trace(random_vector(cfg.n), &layers);
        let outer_input_len = cfg.n;
        let corrupted_trace = corrupted_trace_at_positions(&trace, 0, &[outer_input_len]);

        assert!(!verify_trace_fully(&corrupted_trace, &layers));
    }

    #[test]
    fn random_local_test_rejects_obviously_corrupted_trace_when_sampling_all_rows() {
        let cfg = small_cfg();
        let layers = build_layers(&cfg);
        let trace = encode_with_trace(random_vector(cfg.n), &layers);
        let corrupted_trace = corrupted_trace_at_positions(&trace, trace.layers.len() - 1, &[0]);

        let checks = layers
            .iter()
            .map(|layer| layer.B.rows.len())
            .max()
            .unwrap_or(1);

        assert!(!local_test(&corrupted_trace, &layers, checks));
    }
}
