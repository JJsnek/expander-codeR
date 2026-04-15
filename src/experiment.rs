use crate::field::{F,rand_field};
use ark_ff::Zero;
use crate::encoder::{encode, encode_with_trace, Layer, EncodingTrace};
use crate::expander::{build_layer,SamplingMode};


use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct ExperimentConfig {
    pub n: usize,
    pub alpha_num: usize,
    pub alpha_den: usize,
    pub d: usize,
    pub layers: usize,
    pub trials: usize,
    pub weight:usize,
    pub mode: SamplingMode,
}

//recursion
pub fn build_layers(cfg: &ExperimentConfig) -> Vec<Layer> {
    let mut layers = Vec::new();
    let mut current_n = cfg.n;

    for _ in 0..cfg.layers {
        assert!(current_n % 2 == 0, "n must be divisible by 2 at every layer");

        let projected_n = current_n / 2;

        let layer = build_layer(
            projected_n,  // input size (after projection)
            projected_n,  // output size (keep square for stability)
            cfg.d,
            cfg.mode,
        );

        layers.push(layer);

        // shrink for next layer
        current_n = projected_n;
    }

    layers
}
//approx recursive shrinking from the paper

//random vector gen
use ark_ff::UniformRand;
pub fn random_vector(n:usize)->Vec<F>{
    let mut rng=thread_rng();
    (0..n).map(|_| F::rand(&mut rng)).collect()
}

//sparse vector generator
pub fn random_sparse_vector(n:usize,weight:usize) ->Vec<F>{
    let mut v=vec![F::zero(); n];

    let mut indices: Vec<usize>=(0..n).collect();
    indices.shuffle(&mut thread_rng());

    for &i in indices.iter().take(weight){
        v[i]=rand_field();
    }
    v
}
//corrupt for testing
pub fn corrupt(v: &mut [F], num_errors: usize) {
    use rand::seq::SliceRandom;

    let mut rng = rand::thread_rng();
    let mut indices: Vec<_> = (0..v.len()).collect();
    indices.shuffle(&mut rng);

    for &i in indices.iter().take(num_errors) {
        v[i] = rand_field();
    }
}

//zero check helper
pub fn is_zero_vector(v:&[F])->bool{
    v.iter().all(|x| x.is_zero())
}







//constraints
pub fn local_test(
    trace: &EncodingTrace,
    layers: &[Layer],
    num_checks: usize,
) -> bool {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for (layer_idx, layer) in layers.iter().enumerate() {
        let current = &trace.layers[layer_idx + 1];
        let next = &trace.layers[layer_idx];

        let x_len = next.len() / 2;
        let parity = &next[x_len..];

        for _ in 0..num_checks {
            let row = rng.gen_range(0..layer.B.rows.len());

            let mut acc = F::zero();

            for (col, val) in &layer.B.rows[row] {
                let mut inner_acc = F::zero();

                for (inner_col, inner_val) in &layer.A.rows[*col] {
                  
                    inner_acc += *inner_val * current[*inner_col];
                }

                acc += *val * inner_acc;
            }

            if acc != parity[row] {
                return false;
            }
        }
    }

    true
}

//core experiment
use std::time::Instant;

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

pub fn run_experiment(cfg: &ExperimentConfig)->ExperimentResult{
    let layers=build_layers(cfg);


    let mut failures=0;
    let mut total_time=0.0;

    for _ in 0..cfg.trials{
        let x=random_sparse_vector(cfg.n,cfg.weight);
        //if !printed {println!("\n=== SAMPLE INPUT ===");println!("Input length: {}", x.len());}
        let start=Instant::now();
        let trace = encode_with_trace(x, &layers);
        let elapsed=start.elapsed().as_secs_f64();

        total_time+=elapsed;
        
        let mut corrupted = trace.layers.last().unwrap().clone();
        corrupt(&mut corrupted, cfg.weight);

        //if !printed {println!("\n=== ENCODED OUTPUT ===");println!("Output length: {}", y.len());printed = true;}
        
        // simulate verifier detection
        let detected = !local_test(&trace, &layers, 5);

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
        trials:cfg.trials, 
        failures, 
        avg_time_ms: (total_time/cfg.trials as f64)*1000.0
     }
}







//csv
use std::fs::File;
use std::io::Write;

pub fn write_csv(results:&[ExperimentResult], filename: &str){
    let mut file=File::create(filename).unwrap();

    writeln!(
        file,
        "mode,n,d,layers,weight,trials,failures,failure_rate,avg_time_ms"
    ).unwrap();

    for r in results{
        let failure_rate=r.failures as f64/r.trials as f64;
        
        let mode_str=match r.mode{
            SamplingMode::Random =>"brakedown",
            SamplingMode::NonZero => "spielman",
            SamplingMode::Hybrid=>"hybrid",
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
        ).unwrap();
    }
}

pub fn guess_cd(alpha: f64, rho: f64, delta: f64) -> (usize, usize) {
    // base scaling from delta (main driver)
    let base_c = (4.0 + delta * 80.0) as usize;   // grows with δ
    let base_d = (18.0 - delta * 80.0) as usize;  // shrinks with δ

    // adjust with α/ρ ratio (outer cost weight)
    let k = alpha / rho;

    let d = ((base_d as f64) * k).round().max(6.0) as usize;
    let c = ((base_c as f64) * (1.0 / k)).round().max(2.0) as usize;

    (c, d)
}