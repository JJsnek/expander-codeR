//! Binary entrypoint for the prototype experiments.
//!
//! The executable first runs an interactive demo, then performs three batches of
//! offline measurements and writes the results to CSV files in the repository
//! root.

use expander_codeR::demo::run_demo;
use expander_codeR::expander::SamplingMode;
use expander_codeR::experiment::*;
use std::fs::File;
use std::io::Write;

/// Convert a floating-point alpha value into a coarse rational approximation.
///
/// The current experiments store alpha as `(numerator, denominator)` inside the
/// config even though most of the code only uses the original floating-point
/// value while constructing the sweep.
fn alpha_to_ratio(alpha: f64) -> (usize, usize) {
    let den = 1000;
    let num = (alpha * den as f64) as usize;
    (num.max(1), den)
}

fn main() {
    // Start with a human-readable demo before launching longer sweeps.
    run_demo();

    let mut results = Vec::new();
    let n = 2048;

    let modes = vec![
        SamplingMode::Random,  // brakedown-style
        SamplingMode::NonZero, // spielman-style
        SamplingMode::Hybrid,
    ];

    let params = vec![
        (0.120, 0.704, 0.02),
        (0.138, 0.680, 0.03),
        (0.178, 0.657, 0.04),
        (0.200, 0.610, 0.05),
        (0.211, 0.619, 0.06),
        (0.238, 0.581, 0.07),
    ];

    // Sweep a small table of alpha/rho/delta values across all sampling modes.
    for &(alpha, rho, delta) in &params {
        let (weight, d) = guess_cd(alpha, rho, delta);
        let (alpha_num, alpha_den) = alpha_to_ratio(alpha);

        for &mode in &modes {
            let cfg = ExperimentConfig {
                n,
                alpha_num,
                alpha_den,
                d,
                layers: 2,
                trials: 200,
                weight,
                mode,
            };

            println!(
                "α={:.3}, ρ={:.3}, δ={:.3} → (c,d)=({}, {}) mode={:?}",
                alpha, rho, delta, weight, d, mode
            );

            let result = run_experiment(&cfg);
            results.push(result);
        }
    }

    write_csv(&results, "results.csv");

    println!("\n=== N SCALING EXPERIMENT ===");

    let mut n_results = Vec::new();

    // Hold one plausible `(c, d)` pair fixed and observe scaling in `n`.
    let weight = 6;
    let d = 11;
    let ns = vec![512, 1024, 2048, 4096];

    let modes = vec![
        SamplingMode::Random,
        SamplingMode::NonZero,
        SamplingMode::Hybrid,
    ];

    for &n in &ns {
        for &mode in &modes {
            let cfg = ExperimentConfig {
                n,
                alpha_num: 1,
                alpha_den: 3,
                d,
                layers: 2,
                trials: 100,
                weight,
                mode,
            };

            println!(
                "SCALING → mode={:?}, n={}, (c,d)=({}, {})",
                mode, n, weight, d
            );

            let result = run_experiment(&cfg);
            n_results.push(result);
        }
    }

    write_csv(&n_results, "results_n_scaling.csv");

    println!("\n=== COST MODEL VALIDATION ===");

    let mut cost_results = Vec::new();
    let n = 2048;

    // Hold the mode fixed and compare runtime against a lightweight cost model.
    let mode = SamplingMode::Hybrid;

    let params = vec![
        (0.120, 0.704, 0.02),
        (0.178, 0.657, 0.04),
        (0.238, 0.581, 0.07),
    ];

    let cd_sweep = vec![(4, 8), (6, 11), (8, 12), (12, 16)];

    for &(alpha, rho, _delta) in &params {
        for &(c, d) in &cd_sweep {
            let (alpha_num, alpha_den) = alpha_to_ratio(alpha);

            let cfg = ExperimentConfig {
                n,
                alpha_num,
                alpha_den,
                d,
                layers: 2,
                trials: 150,
                weight: c,
                mode,
            };

            let start = std::time::Instant::now();
            let _result = run_experiment(&cfg);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;

            let cost_model_value = (1.0 / (1.0 - alpha)) * (c as f64 + (alpha / rho) * d as f64);

            cost_results.push((alpha, rho, c, d, elapsed, cost_model_value));
        }
    }

    let mut file = File::create("results_cost_model.csv").unwrap();
    writeln!(file, "alpha,rho,c,d,measured_ms,model_value").unwrap();

    for r in cost_results {
        writeln!(file, "{},{},{},{},{},{}", r.0, r.1, r.2, r.3, r.4, r.5).unwrap();
    }
}
