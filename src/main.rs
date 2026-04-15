use std::fs::File; use std::io::Write;
use expander_codeR::expander::SamplingMode;
use expander_codeR::experiment::*;
use std::io;
use expander_codeR::demo::run_demo;



fn alpha_to_ratio(alpha: f64) -> (usize, usize) {
    let den = 1000;
    let num = (alpha * den as f64) as usize;
    (num.max(1), den)
}


fn main() {

    run_demo();



    let mut results = Vec::new();

    let n = 2048;

    // Table 1-style configs: (c_n ≈ weight, d_n ≈ d)
    

    let modes = vec![
        SamplingMode::Random,   // brakedown
        SamplingMode::NonZero,  // spielman
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

// fix one good config (where hybrid showed advantage)
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
            alpha_den: 3, // fixed alpha scaling test
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

// write separate file
write_csv(&n_results, "results_n_scaling.csv");






println!("\n=== COST MODEL VALIDATION ===");

let mut cost_results = Vec::new();

let n = 2048;

// FIXED modes (hybrid is best here)
let mode = SamplingMode::Hybrid;

// FIX alpha, rho pairs from your existing dataset
let params = vec![
    (0.120, 0.704, 0.02),
    (0.178, 0.657, 0.04),
    (0.238, 0.581, 0.07),
];

// SWEEP c,d explicitly (IMPORTANT)
let cd_sweep = vec![
    (4, 8),
    (6, 11),
    (8, 12),
    (12, 16),
];

for &(alpha, rho, delta) in &params {
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
        let result = run_experiment(&cfg);
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        let cost_model_value =
            (1.0 / (1.0 - alpha)) *
            (c as f64 + (alpha / rho) * d as f64);

        cost_results.push((
            alpha,
            rho,
            c,
            d,
            elapsed,
            cost_model_value,
        ));
    }
}

let mut file = File::create("results_cost_model.csv").unwrap();

writeln!(file, "alpha,rho,c,d,measured_ms,model_value").unwrap();

for r in cost_results {
    writeln!(
        file,
        "{},{},{},{},{},{}",
        r.0, r.1, r.2, r.3, r.4, r.5
    ).unwrap();
}

}