//! Interactive demo for inspecting one small encoding instance.
//!
//! The demo is intentionally verbose: it prints the sampled input, the full
//! recursive trace, a corrupted final output, and then runs a lightweight
//! sampling-based check.

use std::io;

use crate::encoder::encode_with_trace;
use crate::expander::SamplingMode;
use crate::experiment::{ExperimentConfig, build_layers, corrupt, random_vector};
use crate::field::F;

/// Print up to `max` entries from a vector with a simple label.
fn print_vector(label: &str, v: &[F], max: usize) {
    println!("{}", label);

    for (i, val) in v.iter().take(max).enumerate() {
        println!("  [{}] = {}", i, val);
    }

    if v.len() > max {
        println!("  ... ({} more elements)", v.len() - max);
    }
}

/// Prompt the user for the demo input length.
fn read_n() -> usize {
    let mut input = String::new();

    println!("Enter n (e.g. 512, 1024, 2048):");
    io::stdin().read_line(&mut input).unwrap();

    input.trim().parse().expect("Invalid number")
}

/// Run the interactive demo and print a human-readable encoding trace.
pub fn run_demo() {
    let n = read_n();

    let cfg = ExperimentConfig {
        n,
        alpha_num: 1,
        alpha_den: 3,
        d: 11,
        layers: 2,
        trials: 1,
        weight: 6,
        mode: SamplingMode::Hybrid,
    };

    let layers = build_layers(&cfg);

    println!("\n=== DEMO MODE ===");

    let trials = 20;
    let mut detected_count = 0;

    for i in 0..trials {
        let x = random_vector(n);
        let trace = encode_with_trace(x.clone(), &layers);

        println!("\n=== INPUT VECTOR ===");
        print_vector("x:", &x, 10);

        println!("\n=== ENCODING TRACE ===");
        for (layer_idx, layer_out) in trace.layers.iter().enumerate() {
            print_vector(&format!("Layer {}", layer_idx), layer_out, 10);
        }

        // The final trace entry is the innermost output produced by the
        // recursion. The demo corrupts a copy for illustration.
        let correct = trace.layers.last().unwrap().clone();
        let mut y = correct.clone();
        corrupt(&mut y, n / 10);

        println!("\n=== FINAL OUTPUT (CORRECT) ===");
        print_vector("y_correct:", &correct, 5);

        println!("\n=== FINAL OUTPUT (CORRUPTED) ===");
        print_vector("y_corrupted:", &y, 5);

        let detected = !demo_verify_sampling(&x, &y, &layers, 5);

        if detected {
            detected_count += 1;
            println!("Trial {} → DETECTED", i);
        } else {
            println!("Trial {} → FAILED", i);
        }
    }

    println!("\nDetection rate: {}/{}", detected_count, trials);
}

/// Demo-only verifier that samples a few output coordinates.
///
/// This function recomputes a compact output by applying each layer's `A` then
/// `B` maps in sequence. That is simpler than the actual recursive encoder used
/// elsewhere in the crate, so this routine should be read as an illustrative demo
/// check rather than a faithful verifier for the full construction.
pub fn demo_verify_sampling(
    x: &[F],
    y: &[F],
    layers: &[crate::encoder::Layer],
    num_checks: usize,
) -> bool {
    use rand::Rng;
    use rand::thread_rng;

    let mut rng = thread_rng();
    let mut current = x.to_vec();

    // Propagate through the sparse linear maps and sample a few output indices.
    for layer in layers {
        let inner = crate::encoder::apply_matrix(&current, &layer.A);
        current = crate::encoder::apply_matrix(&inner, &layer.B);
    }

    for _ in 0..num_checks {
        let i = rng.gen_range(0..current.len());

        if current[i] != y[i] {
            return false;
        }
    }

    true
}
