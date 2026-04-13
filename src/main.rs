
use expander_codeR::expander::SamplingMode;
use expander_codeR::experiment::*;

fn main() {
    let mut results = Vec::new();

    let n = 2048;

    // Table 1-style configs: (c_n ≈ weight, d_n ≈ d)
    let configs = vec![
        (4, 7),
        (4, 9),
        (6, 9),
        (6, 11),
        (8, 11),
    ];

    let modes = vec![
        SamplingMode::Random,   // brakedown
        SamplingMode::NonZero,  // spielman
        SamplingMode::Hybrid,
    ];

    for &(weight, d) in &configs {
        for &mode in &modes {
            let cfg = ExperimentConfig {
                n,
                m: n / 2,        // will be adapted by recursion
                d,
                layers: 2,       // try 3 later if stable
                trials: 200,     // reduce noise but keep runtime reasonable
                weight,          // ✅ FIXED (you had weight:2 before!)
                mode,
            };

            println!(
                "mode={:?}, (c,d)=({},{}), n={}",
                mode, weight, d, n
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
            m: n / 2,
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
}