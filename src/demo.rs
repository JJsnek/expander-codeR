use std::io;
use rand::Rng;

use crate::field::F;
use crate::encoder::{encode_with_trace, Layer};
use crate::experiment::{
    random_vector,
    corrupt,
    local_test,
    build_layers,
    ExperimentConfig,
};
use crate::expander::SamplingMode;


fn read_n() -> usize {
    let mut input = String::new();

    println!("Enter n (e.g. 512, 1024, 2048):");

    io::stdin().read_line(&mut input).unwrap();

    input.trim().parse().expect("Invalid number")
}

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

    let mut y = trace.layers.last().unwrap().clone();
    corrupt(&mut y, n / 10);

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


pub fn demo_verify_sampling(
    x: &[F],
    y: &[F],
    layers: &[crate::encoder::Layer],
    num_checks: usize,
) -> bool {
    use rand::thread_rng;
    use rand::Rng;

    let mut rng = thread_rng();

    let mut current = x.to_vec();

    // propagate through layers
    for layer in layers.iter() {
        let inner = crate::encoder::apply_matrix(&current, &layer.A);
        current = crate::encoder::apply_matrix(&inner, &layer.B);
    }

    // now current == expected final output

    for _ in 0..num_checks {
        let i = rng.gen_range(0..current.len());

        if current[i] != y[i] {
            return false; // detected corruption
        }
    }

    true
}