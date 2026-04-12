use crate::field::{F,rand_field};
use ark_ff::Zero;
use crate::encoder::{encode_recursive,Layer};
use crate::expander::{build_layer,SamplingMode};


use rand::seq::SliceRandom;
use rand::thread_rng;

pub struct ExperimentConfig {
    pub n: usize,
    pub m: usize,
    pub d: usize,
    pub layers: usize,
    pub trials: usize,
    pub weight:usize,
    pub mode: SamplingMode,
}

//recursion

pub fn build_layers(cfg: &ExperimentConfig) -> Vec<Layer> {
    let mut layers = Vec::new();

    let mut n = cfg.n;
    let mut m = cfg.m;

    for i in 0..cfg.layers {
        let mode = match cfg.mode {
            SamplingMode::Random => SamplingMode::Random,
            SamplingMode::NonZero => SamplingMode::NonZero,
            SamplingMode::Hybrid => {
                if i < cfg.layers / 2 {
                    SamplingMode::NonZero
                } else {
                    SamplingMode::Random
                }
            }
        };

        let layer = build_layer(n, m, cfg.d, mode);
        layers.push(layer);

        n = m;
        m /= 2;
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

//sprarse vector generator
pub fn random_sparse_vector(n:usize,weight:usize) ->Vec<F>{
    let mut v=vec![F::zero(); n];

    let mut indices: Vec<usize>=(0..n).collect();
    indices.shuffle(&mut thread_rng());

    for &i in indices.iter().take(weight){
        v[i]=rand_field();
    }
    v
}


//zero check helper
pub fn is_zero_vector(v:&[F])->bool{
    v.iter().all(|x| x.is_zero())
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

        let start=Instant::now();
        let y= encode_recursive(x,&layers);
        let elapsed=start.elapsed().as_secs_f64();

        total_time+=elapsed;

        if is_zero_vector(&y){
            failures +=1;
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