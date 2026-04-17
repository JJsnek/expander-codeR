//! Interactive demo for inspecting one small encoding instance.
//!
//! The demo is intentionally verbose: it prints the sampled input, the full
//! recursive trace, a corrupted final output, and then runs a lightweight
//! sampling-based check.
//!
//! Miloš requested that the demo verifier stop pretending a separately computed
//! output is the object under test. The demo now corrupts a recorded trace and
//! verifies that exact trace.

use std::io;

use crate::encoder::encode_with_trace;
use crate::expander::SamplingMode;
use crate::experiment::{
    ExperimentConfig, build_layers, corrupt_trace_layer, random_vector, verify_trace_fully,
};
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

        // Suggestion from Miloš: the demo should present and verify the same
        // corrupted object. We therefore mutate the innermost trace layer and
        // show that exact corrupted layer to the user.
        let correct = trace.layers.last().unwrap().clone();
        let corrupted_trace = corrupt_trace_layer(&trace, trace.layers.len() - 1, n / 10);
        let y = corrupted_trace.layers.last().unwrap().clone();

        println!("\n=== FINAL OUTPUT (CORRECT) ===");
        print_vector("y_correct:", &correct, 5);

        println!("\n=== FINAL OUTPUT (CORRUPTED) ===");
        print_vector("y_corrupted:", &y, 5);

        let detected = !demo_verify_sampling(&corrupted_trace, &layers);

        if detected {
            detected_count += 1;
            println!("Trial {} → DETECTED", i);
        } else {
            println!("Trial {} → FAILED", i);
        }
    }

    println!("\nDetection rate: {}/{}", detected_count, trials);
}

/// Demo verifier for the recursive construction.
///
/// The demo uses the exhaustive verifier so the user sees an unambiguous
/// pass/fail outcome for the exact corrupted trace that was printed.
pub fn demo_verify_sampling(
    trace: &crate::encoder::EncodingTrace,
    layers: &[crate::encoder::Layer],
) -> bool {
    verify_trace_fully(trace, layers)
}
